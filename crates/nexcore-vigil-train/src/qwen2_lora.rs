//! Qwen2 model with optional LoRA adapters at the four attention projections.
//!
//! This file is a fork of `candle_transformers::models::qwen2` (Candle 0.9.2)
//! with LoRA injection hooks wired into the `Attention` layer. The diff from
//! upstream is deliberately small:
//! - Imports repointed (`crate::` → external paths).
//! - `Attention` gains four `Option<Arc<LoraAdapter>>` fields.
//! - `Attention::new_with_lora` constructor accepts optional adapters.
//! - `Attention::forward` adds `adapter.apply(xs)` after each projection.
//! - `Model::new_with_lora` / `ModelForCausalLM::new_with_lora` plumb the
//!   bundle through the layer stack.

#![allow(dead_code)]

use crate::lora::{LoraAdapter, LoraBundle, LoraTarget};
use candle_core::{D, DType, Device, IndexOp, Module, Result, Tensor};
use candle_nn::{Activation, VarBuilder};
use candle_transformers::models::with_tracing::{Linear, RmsNorm, linear, linear_no_bias};
use candle_transformers::utils::repeat_kv;
use std::sync::Arc;

/// Add LoRA delta to a base projection output. If no adapter, passes through.
/// Bridges `LoraAdapter::apply` (returns `nexcore_error::Result`) to Candle's
/// `Result` via `candle_core::Error::wrap`.
fn add_lora(base_out: Tensor, adapter: Option<&LoraAdapter>, x: &Tensor) -> Result<Tensor> {
    match adapter {
        None => Ok(base_out),
        Some(a) => {
            let delta = a
                .apply(x)
                .map_err(|e| candle_core::Error::Msg(format!("lora apply: {e}")))?;
            base_out + delta
        }
    }
}

// Re-export the upstream Config so callers have a single canonical type.
// This avoids a fork-drift bug where two structurally-identical Config types
// can't be passed to each other's constructors.
pub use candle_transformers::models::qwen2::Config;

#[derive(Debug, Clone)]
struct RotaryEmbedding {
    sin: Tensor,
    cos: Tensor,
}

impl RotaryEmbedding {
    fn new(dtype: DType, cfg: &Config, dev: &Device) -> Result<Self> {
        let dim = cfg.hidden_size / cfg.num_attention_heads;
        let max_seq_len = cfg.max_position_embeddings;
        let inv_freq: Vec<_> = (0..dim)
            .step_by(2)
            .map(|i| 1f32 / cfg.rope_theta.powf(i as f64 / dim as f64) as f32)
            .collect();
        let inv_freq_len = inv_freq.len();
        let inv_freq = Tensor::from_vec(inv_freq, (1, inv_freq_len), dev)?.to_dtype(dtype)?;
        let t = Tensor::arange(0u32, max_seq_len as u32, dev)?
            .to_dtype(dtype)?
            .reshape((max_seq_len, 1))?;
        let freqs = t.matmul(&inv_freq)?;
        Ok(Self {
            sin: freqs.sin()?,
            cos: freqs.cos()?,
        })
    }

    fn apply_rotary_emb_qkv(
        &self,
        q: &Tensor,
        k: &Tensor,
        seqlen_offset: usize,
    ) -> Result<(Tensor, Tensor)> {
        let (_b_sz, _h, seq_len, _n_embd) = q.dims4()?;
        let cos = self.cos.narrow(0, seqlen_offset, seq_len)?;
        let sin = self.sin.narrow(0, seqlen_offset, seq_len)?;
        let q_embed = candle_nn::rotary_emb::rope(&q.contiguous()?, &cos, &sin)?;
        let k_embed = candle_nn::rotary_emb::rope(&k.contiguous()?, &cos, &sin)?;
        Ok((q_embed, k_embed))
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
struct MLP {
    gate_proj: Linear,
    up_proj: Linear,
    down_proj: Linear,
    act_fn: Activation,
}

impl MLP {
    fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let hidden_sz = cfg.hidden_size;
        let intermediate_sz = cfg.intermediate_size;
        let gate_proj = linear_no_bias(hidden_sz, intermediate_sz, vb.pp("gate_proj"))?;
        let up_proj = linear_no_bias(hidden_sz, intermediate_sz, vb.pp("up_proj"))?;
        let down_proj = linear_no_bias(intermediate_sz, hidden_sz, vb.pp("down_proj"))?;
        Ok(Self {
            gate_proj,
            up_proj,
            down_proj,
            act_fn: cfg.hidden_act,
        })
    }
}

impl Module for MLP {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let lhs = xs.apply(&self.gate_proj)?.apply(&self.act_fn)?;
        let rhs = xs.apply(&self.up_proj)?;
        (lhs * rhs)?.apply(&self.down_proj)
    }
}

#[derive(Debug, Clone)]
struct Attention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    o_proj: Linear,
    // LoRA adapter slots. `None` = no delta added at this projection.
    lora_q: Option<Arc<LoraAdapter>>,
    lora_k: Option<Arc<LoraAdapter>>,
    lora_v: Option<Arc<LoraAdapter>>,
    lora_o: Option<Arc<LoraAdapter>>,
    num_heads: usize,
    num_kv_heads: usize,
    num_kv_groups: usize,
    head_dim: usize,
    hidden_size: usize,
    rotary_emb: Arc<RotaryEmbedding>,
    kv_cache: Option<(Tensor, Tensor)>,
}

impl Attention {
    fn new(rotary_emb: Arc<RotaryEmbedding>, cfg: &Config, vb: VarBuilder) -> Result<Self> {
        Self::new_with_lora(rotary_emb, cfg, vb, None, None, None, None)
    }

    fn new_with_lora(
        rotary_emb: Arc<RotaryEmbedding>,
        cfg: &Config,
        vb: VarBuilder,
        lora_q: Option<Arc<LoraAdapter>>,
        lora_k: Option<Arc<LoraAdapter>>,
        lora_v: Option<Arc<LoraAdapter>>,
        lora_o: Option<Arc<LoraAdapter>>,
    ) -> Result<Self> {
        let hidden_sz = cfg.hidden_size;
        let num_heads = cfg.num_attention_heads;
        let num_kv_heads = cfg.num_key_value_heads;
        let num_kv_groups = num_heads / num_kv_heads;
        let head_dim = hidden_sz / num_heads;
        let q_proj = linear(hidden_sz, num_heads * head_dim, vb.pp("q_proj"))?;
        let k_proj = linear(hidden_sz, num_kv_heads * head_dim, vb.pp("k_proj"))?;
        let v_proj = linear(hidden_sz, num_kv_heads * head_dim, vb.pp("v_proj"))?;
        let o_proj = linear_no_bias(num_heads * head_dim, hidden_sz, vb.pp("o_proj"))?;
        Ok(Self {
            q_proj,
            k_proj,
            v_proj,
            o_proj,
            lora_q,
            lora_k,
            lora_v,
            lora_o,
            num_heads,
            num_kv_heads,
            num_kv_groups,
            head_dim,
            hidden_size: hidden_sz,
            rotary_emb,
            kv_cache: None,
        })
    }

    fn forward(
        &mut self,
        xs: &Tensor,
        attention_mask: Option<&Tensor>,
        seqlen_offset: usize,
    ) -> Result<Tensor> {
        let (b_sz, q_len, _) = xs.dims3()?;

        let query_states = add_lora(self.q_proj.forward(xs)?, self.lora_q.as_deref(), xs)?;
        let key_states = add_lora(self.k_proj.forward(xs)?, self.lora_k.as_deref(), xs)?;
        let value_states = add_lora(self.v_proj.forward(xs)?, self.lora_v.as_deref(), xs)?;

        let query_states = query_states
            .reshape((b_sz, q_len, self.num_heads, self.head_dim))?
            .transpose(1, 2)?;
        let key_states = key_states
            .reshape((b_sz, q_len, self.num_kv_heads, self.head_dim))?
            .transpose(1, 2)?;
        let value_states = value_states
            .reshape((b_sz, q_len, self.num_kv_heads, self.head_dim))?
            .transpose(1, 2)?;

        let (query_states, key_states) =
            self.rotary_emb
                .apply_rotary_emb_qkv(&query_states, &key_states, seqlen_offset)?;

        let (key_states, value_states) = match &self.kv_cache {
            None => (key_states, value_states),
            Some((prev_k, prev_v)) => {
                let key_states = Tensor::cat(&[prev_k, &key_states], 2)?;
                let value_states = Tensor::cat(&[prev_v, &value_states], 2)?;
                (key_states, value_states)
            }
        };
        self.kv_cache = Some((key_states.clone(), value_states.clone()));

        let key_states = repeat_kv(key_states, self.num_kv_groups)?.contiguous()?;
        let value_states = repeat_kv(value_states, self.num_kv_groups)?.contiguous()?;

        let attn_output = {
            let scale = 1f64 / f64::sqrt(self.head_dim as f64);
            let attn_weights = (query_states.matmul(&key_states.transpose(2, 3)?)? * scale)?;

            let attn_weights = match attention_mask {
                None => attn_weights,
                Some(mask) => attn_weights.broadcast_add(mask)?,
            };
            let attn_weights = candle_nn::ops::softmax_last_dim(&attn_weights)?;
            attn_weights.matmul(&value_states)?
        };
        let attn_output_reshaped =
            attn_output
                .transpose(1, 2)?
                .reshape((b_sz, q_len, self.hidden_size))?;
        let o_input = attn_output_reshaped.clone();
        let o_out = self.o_proj.forward(&attn_output_reshaped)?;
        add_lora(o_out, self.lora_o.as_deref(), &o_input)
    }

    fn clear_kv_cache(&mut self) {
        self.kv_cache = None
    }
}

#[derive(Debug, Clone)]
struct DecoderLayer {
    self_attn: Attention,
    mlp: MLP,
    input_layernorm: RmsNorm,
    post_attention_layernorm: RmsNorm,
}

impl DecoderLayer {
    fn new(rotary_emb: Arc<RotaryEmbedding>, cfg: &Config, vb: VarBuilder) -> Result<Self> {
        Self::new_with_lora(rotary_emb, cfg, vb, None, None, None, None)
    }

    fn new_with_lora(
        rotary_emb: Arc<RotaryEmbedding>,
        cfg: &Config,
        vb: VarBuilder,
        lora_q: Option<Arc<LoraAdapter>>,
        lora_k: Option<Arc<LoraAdapter>>,
        lora_v: Option<Arc<LoraAdapter>>,
        lora_o: Option<Arc<LoraAdapter>>,
    ) -> Result<Self> {
        let self_attn = Attention::new_with_lora(
            rotary_emb,
            cfg,
            vb.pp("self_attn"),
            lora_q,
            lora_k,
            lora_v,
            lora_o,
        )?;
        let mlp = MLP::new(cfg, vb.pp("mlp"))?;
        let input_layernorm =
            RmsNorm::new(cfg.hidden_size, cfg.rms_norm_eps, vb.pp("input_layernorm"))?;
        let post_attention_layernorm = RmsNorm::new(
            cfg.hidden_size,
            cfg.rms_norm_eps,
            vb.pp("post_attention_layernorm"),
        )?;
        Ok(Self {
            self_attn,
            mlp,
            input_layernorm,
            post_attention_layernorm,
        })
    }

    fn forward(
        &mut self,
        xs: &Tensor,
        attention_mask: Option<&Tensor>,
        seqlen_offset: usize,
    ) -> Result<Tensor> {
        let residual = xs;
        // Use `forward_diff` instead of `forward` / `apply` so the slow
        // autograd-safe RmsNorm path runs — candle 0.9.2's default `forward`
        // calls `ops::rms_norm` which uses `apply_op2_no_bwd` (no backward),
        // severing gradient flow through LoRA adapters.  See task #5.
        let xs = self.input_layernorm.forward_diff(xs)?;
        let xs = self.self_attn.forward(&xs, attention_mask, seqlen_offset)?;
        let xs = (xs + residual)?;
        let residual = &xs;
        let xs = self.post_attention_layernorm.forward_diff(&xs)?.apply(&self.mlp)?;
        residual + xs
    }

    fn clear_kv_cache(&mut self) {
        self.self_attn.clear_kv_cache()
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    embed_tokens: candle_nn::Embedding,
    layers: Vec<DecoderLayer>,
    norm: RmsNorm,
    sliding_window: usize,
    device: Device,
    dtype: DType,
}

impl Model {
    pub fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        Self::new_with_lora(cfg, vb, None)
    }

    /// Construct with optional LoRA bundle. If `bundle` is `Some`, each layer's
    /// Attention receives the 4 adapters matching (layer_idx, {q,k,v,o}_proj).
    pub fn new_with_lora(
        cfg: &Config,
        vb: VarBuilder,
        bundle: Option<&LoraBundle>,
    ) -> Result<Self> {
        let vb_m = vb.pp("model");
        let embed_tokens =
            candle_nn::embedding(cfg.vocab_size, cfg.hidden_size, vb_m.pp("embed_tokens"))?;
        let rotary_emb = Arc::new(RotaryEmbedding::new(vb.dtype(), cfg, vb_m.device())?);
        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        let vb_l = vb_m.pp("layers");
        for layer_idx in 0..cfg.num_hidden_layers {
            let (lq, lk, lv, lo) = match bundle {
                Some(b) => (
                    b.find_arc(layer_idx, LoraTarget::QProj),
                    b.find_arc(layer_idx, LoraTarget::KProj),
                    b.find_arc(layer_idx, LoraTarget::VProj),
                    b.find_arc(layer_idx, LoraTarget::OProj),
                ),
                None => (None, None, None, None),
            };
            let layer = DecoderLayer::new_with_lora(
                rotary_emb.clone(),
                cfg,
                vb_l.pp(layer_idx),
                lq,
                lk,
                lv,
                lo,
            )?;
            layers.push(layer)
        }
        let norm = RmsNorm::new(cfg.hidden_size, cfg.rms_norm_eps, vb_m.pp("norm"))?;
        Ok(Self {
            embed_tokens,
            layers,
            norm,
            sliding_window: cfg.sliding_window,
            device: vb.device().clone(),
            dtype: vb.dtype(),
        })
    }

    fn prepare_causal_attention_mask(
        &self,
        b_size: usize,
        tgt_len: usize,
        seqlen_offset: usize,
    ) -> Result<Tensor> {
        // Sliding window mask?
        let mask: Vec<_> = (0..tgt_len)
            .flat_map(|i| {
                (0..tgt_len).map(move |j| {
                    if i < j || j + self.sliding_window < i {
                        f32::NEG_INFINITY
                    } else {
                        0.
                    }
                })
            })
            .collect();
        let mask = Tensor::from_slice(&mask, (tgt_len, tgt_len), &self.device)?;
        let mask = if seqlen_offset > 0 {
            let mask0 = Tensor::zeros((tgt_len, seqlen_offset), self.dtype, &self.device)?;
            Tensor::cat(&[&mask0, &mask], D::Minus1)?
        } else {
            mask
        };
        mask.expand((b_size, 1, tgt_len, tgt_len + seqlen_offset))?
            .to_dtype(self.dtype)
    }

    fn prepare_attention_mask(&self, attn_mask: &Tensor) -> Result<Tensor> {
        let (b_sz, sql_len) = attn_mask.dims2()?;
        let mut mask: Vec<Tensor> = vec![];
        for b in 0..b_sz {
            mask.push(attn_mask.i((b, ..))?.expand((1, 1, sql_len, sql_len))?);
        }
        let mask = Tensor::cat(&mask, 0)?;
        let on_true = mask.zeros_like()?.to_dtype(self.dtype)?;
        let on_false = Tensor::new(f32::NEG_INFINITY, &self.device)?
            .broadcast_as(mask.shape())?
            .to_dtype(self.dtype)?;
        mask.where_cond(&on_true, &on_false)
    }

    pub fn forward(
        &mut self,
        input_ids: &Tensor,
        seqlen_offset: usize,
        attn_mask: Option<&Tensor>,
    ) -> Result<Tensor> {
        let (b_size, seq_len) = input_ids.dims2()?;
        let attention_mask: Option<Tensor> = match attn_mask {
            Some(mask) => Some(self.prepare_attention_mask(mask)?),
            None => {
                if seq_len <= 1 {
                    None
                } else {
                    Some(self.prepare_causal_attention_mask(b_size, seq_len, seqlen_offset)?)
                }
            }
        };
        let mut xs = self.embed_tokens.forward(input_ids)?;
        for layer in self.layers.iter_mut() {
            xs = layer.forward(&xs, attention_mask.as_ref(), seqlen_offset)?
        }
        // Fix for task #5: `forward_diff` uses autograd-safe slow path.
        self.norm.forward_diff(&xs)
    }

    pub fn clear_kv_cache(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.clear_kv_cache()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelForCausalLM {
    base_model: Model,
    lm_head: Linear,
}

impl ModelForCausalLM {
    pub fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        Self::new_with_lora(cfg, vb, None)
    }

    /// Construct with optional LoRA bundle. See `Model::new_with_lora`.
    pub fn new_with_lora(
        cfg: &Config,
        vb: VarBuilder,
        bundle: Option<&LoraBundle>,
    ) -> Result<Self> {
        let base_model = Model::new_with_lora(cfg, vb.clone(), bundle)?;
        let lm_head = if vb.contains_tensor("lm_head.weight") {
            linear_no_bias(cfg.hidden_size, cfg.vocab_size, vb.pp("lm_head"))?
        } else {
            Linear::from_weights(base_model.embed_tokens.embeddings().clone(), None)
        };
        Ok(Self {
            base_model,
            lm_head,
        })
    }

    pub fn forward(&mut self, input_ids: &Tensor, seqlen_offset: usize) -> Result<Tensor> {
        let (_b_size, seq_len) = input_ids.dims2()?;
        self.base_model
            .forward(input_ids, seqlen_offset, None)?
            .narrow(1, seq_len - 1, 1)?
            .apply(&self.lm_head)
    }

    /// Full-sequence forward pass returning logits at every position.
    /// Required for training: SFT loss is computed across all assistant tokens.
    pub fn forward_all(&mut self, input_ids: &Tensor, seqlen_offset: usize) -> Result<Tensor> {
        self.base_model
            .forward(input_ids, seqlen_offset, None)?
            .apply(&self.lm_head)
    }

    pub fn clear_kv_cache(&mut self) {
        self.base_model.clear_kv_cache()
    }
}
