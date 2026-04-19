//! LoRA adapter — Phase R2.
//!
//! LoRA (Hu et al. 2021) replaces a dense update `W' = W + ΔW` with a low-rank
//! factorization `ΔW = (α/r) · B · A` where `A ∈ ℝ^{r × in_features}` and
//! `B ∈ ℝ^{out_features × r}`. For Qwen2, we attach LoRA to four projections
//! per attention block: `q_proj`, `k_proj`, `v_proj`, `o_proj`.
//!
//! In training, only `A` and `B` have grad; the base weights are frozen.
//! Forward pass: `delta(x) = (α/r) · x · Aᵀ · Bᵀ` (linear math, no bias).
//! Merge (post-training): `W_merged = W_base + (α/r) · B · A`.

use candle_core::{DType, Device, Tensor};
use candle_nn::init::Init;
use candle_nn::{VarBuilder, VarMap};
use nexcore_error::{NexError, Result};

/// LoRA hyperparameters.
#[derive(Clone, Copy, Debug)]
pub struct LoraConfig {
    pub r: usize,
    pub alpha: f64,
    pub dropout: f64,
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            r: 8,
            alpha: 16.0,
            dropout: 0.05,
        }
    }
}

impl LoraConfig {
    /// Scaling factor `α / r`.
    pub fn scaling(&self) -> f64 {
        self.alpha / self.r as f64
    }
}

/// Target projection within a Qwen2 attention block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoraTarget {
    QProj,
    KProj,
    VProj,
    OProj,
}

impl LoraTarget {
    pub fn name(&self) -> &'static str {
        match self {
            LoraTarget::QProj => "q_proj",
            LoraTarget::KProj => "k_proj",
            LoraTarget::VProj => "v_proj",
            LoraTarget::OProj => "o_proj",
        }
    }

    /// Default 4-target set used throughout this crate.
    pub fn canonical_set() -> [LoraTarget; 4] {
        [
            LoraTarget::QProj,
            LoraTarget::KProj,
            LoraTarget::VProj,
            LoraTarget::OProj,
        ]
    }
}

/// A single LoRA adapter: `A ∈ ℝ^{r × in_features}` and `B ∈ ℝ^{out_features × r}`.
#[derive(Debug, Clone)]
pub struct LoraAdapter {
    pub a: Tensor,
    pub b: Tensor,
    pub cfg: LoraConfig,
    pub target: LoraTarget,
    pub layer: usize,
    pub in_features: usize,
    pub out_features: usize,
}

impl LoraAdapter {
    /// Initialize `A` (normal, stdev=1/sqrt(r)) and `B` (zero) so the initial
    /// forward pass is identical to the base model. Zero-init of `B` is the
    /// LoRA paper's choice — training starts from the base model's behavior.
    ///
    /// Both tensors are registered in the `VarBuilder`'s backing `VarMap` under
    /// names `lora.{layer}.{target}.a` and `lora.{layer}.{target}.b`, so the
    /// optimizer can discover them via `VarMap::all_vars()`.
    pub fn init(
        in_features: usize,
        out_features: usize,
        cfg: LoraConfig,
        target: LoraTarget,
        layer: usize,
        vb: &VarBuilder,
    ) -> Result<Self> {
        let prefix = vb.pp(format!("lora.{}.{}", layer, target.name()));
        let stdev = 1.0 / (cfg.r as f64).sqrt();
        let a = prefix
            .get_with_hints((cfg.r, in_features), "a", Init::Randn { mean: 0.0, stdev })
            .map_err(|e| NexError::new(format!("lora A init L{layer}/{}: {e}", target.name())))?;
        let b = prefix
            .get_with_hints((out_features, cfg.r), "b", Init::Const(0.0))
            .map_err(|e| NexError::new(format!("lora B init L{layer}/{}: {e}", target.name())))?;
        Ok(Self {
            a,
            b,
            cfg,
            target,
            layer,
            in_features,
            out_features,
        })
    }

    /// Apply the adapter delta on `x` of shape `[..., in_features]`.
    /// Returns a tensor of shape `[..., out_features]`.
    ///
    /// Math: `delta = (α/r) · x @ Aᵀ @ Bᵀ`.
    ///
    /// Candle's `matmul` requires both operands to have the same rank (both
    /// 3D with broadcastable batch, or both 2D). A/B are 2D, so we flatten
    /// the input's leading dims, matmul as 2D, and reshape back.
    pub fn apply(&self, x: &Tensor) -> Result<Tensor> {
        let dims = x.dims();
        if dims.is_empty() {
            return Err(NexError::new("lora apply: scalar input not supported"));
        }
        let in_features = *dims
            .last()
            .ok_or_else(|| NexError::new("lora apply: empty dims"))?;
        if in_features != self.in_features {
            return Err(NexError::new(format!(
                "lora apply: last dim {} != adapter in_features {}",
                in_features, self.in_features
            )));
        }
        let leading: usize = dims[..dims.len() - 1].iter().product();

        // Flatten to [leading, in_features] for 2D matmul compatibility.
        let x2 = x
            .reshape((leading, in_features))
            .map_err(|e| NexError::new(format!("lora apply reshape2d: {e}")))?;

        // A has shape [r, in]; A^T has shape [in, r]; x2 @ A^T = [leading, r].
        let a_t = self
            .a
            .t()
            .map_err(|e| NexError::new(format!("lora apply a.t: {e}")))?;
        let xa = x2
            .matmul(&a_t)
            .map_err(|e| NexError::new(format!("lora apply x@A^T: {e}")))?;

        // B has shape [out, r]; B^T has shape [r, out]; (x2 @ A^T) @ B^T = [leading, out].
        let b_t = self
            .b
            .t()
            .map_err(|e| NexError::new(format!("lora apply b.t: {e}")))?;
        let xab = xa
            .matmul(&b_t)
            .map_err(|e| NexError::new(format!("lora apply (xA)@B^T: {e}")))?;

        // Reshape back to [..., out_features].
        let mut out_shape: Vec<usize> = dims[..dims.len() - 1].to_vec();
        out_shape.push(self.out_features);
        let xab = xab
            .reshape(out_shape)
            .map_err(|e| NexError::new(format!("lora apply restore shape: {e}")))?;

        let scaled = xab
            .affine(self.cfg.scaling(), 0.0)
            .map_err(|e| NexError::new(format!("lora apply scale: {e}")))?;
        Ok(scaled)
    }

    /// Fold the adapter into a base weight matrix (post-training merge):
    /// `W_merged = W_base + (α/r) · B · A`.
    /// `base_weight` must have shape `[out_features, in_features]`.
    pub fn fold_into(&self, base_weight: &Tensor) -> Result<Tensor> {
        // B @ A has shape [out, in] matching base_weight.
        let ba = self
            .b
            .matmul(&self.a)
            .map_err(|e| NexError::new(format!("lora fold B@A: {e}")))?;
        let scaled = ba
            .affine(self.cfg.scaling(), 0.0)
            .map_err(|e| NexError::new(format!("lora fold scale: {e}")))?;
        let merged =
            (base_weight + scaled).map_err(|e| NexError::new(format!("lora fold add: {e}")))?;
        Ok(merged)
    }

    /// Number of trainable parameters in this adapter: `r · (in + out)`.
    pub fn param_count(&self) -> usize {
        self.cfg.r * (self.in_features + self.out_features)
    }
}

/// Collection of adapters indexed by layer × target. The backing `VarMap` is
/// what the optimizer updates. Adapters are `Arc`-wrapped so the forked Qwen2
/// model can hold shared references into each layer's attention.
pub struct LoraBundle {
    pub adapters: Vec<std::sync::Arc<LoraAdapter>>,
    pub cfg: LoraConfig,
    pub varmap: VarMap,
}

impl LoraBundle {
    /// Attach LoRA adapters to every (layer, target) of a Qwen2 model.
    ///
    /// Qwen2 uses grouped-query attention, so `q_proj` and `o_proj` use
    /// `hidden_size` for both in/out (when considering per-head × num_heads),
    /// but `k_proj`/`v_proj` use `num_key_value_heads × head_dim` for output.
    /// We compute dims from the full config and pass explicit sizes.
    pub fn attach_canonical(
        num_layers: usize,
        hidden_size: usize,
        num_attention_heads: usize,
        num_key_value_heads: usize,
        cfg: LoraConfig,
        dtype: DType,
        device: &Device,
    ) -> Result<Self> {
        if num_attention_heads == 0 {
            return Err(NexError::new("num_attention_heads must be > 0"));
        }
        let head_dim = hidden_size / num_attention_heads;
        let kv_dim = num_key_value_heads * head_dim;

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, dtype, device);

        let mut adapters: Vec<std::sync::Arc<LoraAdapter>> = Vec::with_capacity(num_layers * 4);
        for layer in 0..num_layers {
            for target in LoraTarget::canonical_set() {
                let (in_f, out_f) = match target {
                    LoraTarget::QProj => (hidden_size, hidden_size),
                    LoraTarget::KProj | LoraTarget::VProj => (hidden_size, kv_dim),
                    LoraTarget::OProj => (hidden_size, hidden_size),
                };
                let adapter = LoraAdapter::init(in_f, out_f, cfg, target, layer, &vb)?;
                adapters.push(std::sync::Arc::new(adapter));
            }
        }

        Ok(Self {
            adapters,
            cfg,
            varmap,
        })
    }

    /// Total trainable-parameter count across all adapters.
    pub fn trainable_params(&self) -> usize {
        self.adapters.iter().map(|a| a.param_count()).sum()
    }

    /// Look up an adapter by (layer, target), returning a cloned `Arc`.
    pub fn find_arc(
        &self,
        layer: usize,
        target: LoraTarget,
    ) -> Option<std::sync::Arc<LoraAdapter>> {
        self.adapters
            .iter()
            .find(|a| a.layer == layer && a.target == target)
            .cloned()
    }

    /// Save all adapter tensors to a single safetensors file at `path`.
    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        self.varmap
            .save(path)
            .map_err(|e| NexError::new(format!("save adapter {}: {e}", path.display())))
    }

    /// Load adapter tensors from a safetensors file into `self.varmap`.
    /// The varmap must already have the expected variables registered
    /// (i.e., call `attach_canonical` first to create the shape).
    pub fn load(&mut self, path: &std::path::Path) -> Result<()> {
        self.varmap
            .load(path)
            .map_err(|e| NexError::new(format!("load adapter {}: {e}", path.display())))
    }

    /// Look up an adapter by (layer, target) and return a borrow into the Arc.
    pub fn find(&self, layer: usize, target: LoraTarget) -> Option<&LoraAdapter> {
        self.adapters
            .iter()
            .find(|a| a.layer == layer && a.target == target)
            .map(std::ops::Deref::deref)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests — validate the math without touching a real model.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use candle_core::Device;

    fn cpu() -> Device {
        Device::Cpu
    }

    #[test]
    fn zero_init_produces_zero_delta() {
        // B is zero-initialized, so apply(x) must be the zero tensor.
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &cpu());
        let cfg = LoraConfig::default();
        let adapter = LoraAdapter::init(16, 24, cfg, LoraTarget::QProj, 0, &vb).expect("init");
        let x = Tensor::ones((2, 16), DType::F32, &cpu()).expect("x");
        let delta = adapter.apply(&x).expect("apply");
        let dims = delta.dims();
        assert_eq!(dims, &[2, 24]);
        let sum = delta
            .abs()
            .expect("abs")
            .sum_all()
            .expect("sum")
            .to_scalar::<f32>()
            .expect("scalar");
        assert!(sum.abs() < 1e-6, "expected zero delta, got abs-sum {sum}");
    }

    #[test]
    fn fold_with_zero_b_is_identity() {
        // B=0 => fold_into returns exactly the base weight.
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &cpu());
        let cfg = LoraConfig::default();
        let adapter = LoraAdapter::init(8, 8, cfg, LoraTarget::OProj, 3, &vb).expect("init");
        let base = Tensor::randn(0f32, 1.0, (8, 8), &cpu()).expect("base");
        let merged = adapter.fold_into(&base).expect("fold");
        let delta = (&base - &merged)
            .expect("sub")
            .abs()
            .expect("abs")
            .sum_all()
            .expect("sum")
            .to_scalar::<f32>()
            .expect("scalar");
        assert!(
            delta < 1e-5,
            "merged deviates from base when B=0: abs-sum {delta}"
        );
    }

    #[test]
    fn scaling_follows_alpha_over_r() {
        let cfg = LoraConfig {
            r: 4,
            alpha: 16.0,
            dropout: 0.0,
        };
        assert!((cfg.scaling() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn bundle_attaches_4_times_num_layers() {
        let bundle =
            LoraBundle::attach_canonical(2, 32, 4, 2, LoraConfig::default(), DType::F32, &cpu())
                .expect("bundle");
        assert_eq!(bundle.adapters.len(), 8);
        assert!(bundle.trainable_params() > 0);
    }
}
