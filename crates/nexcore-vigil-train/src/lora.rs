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

use candle_core::{DType, Device, Tensor, Var};
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
///
/// Holds `Var`s (not cloned `Tensor`s) so `as_tensor()` returns the
/// live-tracked tensor the optimizer updates.  Fixed 2026-04-19 (task #5):
/// prior versions stored `a: Tensor, b: Tensor` as clones from
/// `VarBuilder::get_with_hints`, which were leaf tensors disconnected from
/// the autograd graph once the model forward composed them — backward
/// produced zero gradients for every LoRA var.
#[derive(Debug, Clone)]
pub struct LoraAdapter {
    pub a_var: Var,
    pub b_var: Var,
    pub cfg: LoraConfig,
    pub target: LoraTarget,
    pub layer: usize,
    pub in_features: usize,
    pub out_features: usize,
}

impl LoraAdapter {
    /// Current A tensor (live view of the Var — picks up optimizer updates).
    #[inline]
    pub fn a(&self) -> Tensor {
        self.a_var.as_tensor().clone()
    }
    /// Current B tensor (live view of the Var — picks up optimizer updates).
    #[inline]
    pub fn b(&self) -> Tensor {
        self.b_var.as_tensor().clone()
    }
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
        varmap: &VarMap,
    ) -> Result<Self> {
        let prefix = vb.pp(format!("lora.{}.{}", layer, target.name()));
        let stdev = 1.0 / (cfg.r as f64).sqrt();
        // Register vars in the varmap via VarBuilder.  The returned Tensor is
        // discarded — we'll retrieve live Var references from the varmap next.
        let _a = prefix
            .get_with_hints((cfg.r, in_features), "a", Init::Randn { mean: 0.0, stdev })
            .map_err(|e| NexError::new(format!("lora A init L{layer}/{}: {e}", target.name())))?;
        let _b = prefix
            .get_with_hints((out_features, cfg.r), "b", Init::Const(0.0))
            .map_err(|e| NexError::new(format!("lora B init L{layer}/{}: {e}", target.name())))?;
        let a_name = format!("lora.{}.{}.a", layer, target.name());
        let b_name = format!("lora.{}.{}.b", layer, target.name());
        let data = varmap.data();
        let guard = data
            .lock()
            .map_err(|e| NexError::new(format!("varmap lock: {e}")))?;
        let a_var = guard
            .get(&a_name)
            .ok_or_else(|| NexError::new(format!("var missing after init: {a_name}")))?
            .clone();
        let b_var = guard
            .get(&b_name)
            .ok_or_else(|| NexError::new(format!("var missing after init: {b_name}")))?
            .clone();
        drop(guard);
        Ok(Self {
            a_var,
            b_var,
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
            .a_var
            .as_tensor()
            .t()
            .map_err(|e| NexError::new(format!("lora apply a.t: {e}")))?;
        let xa = x2
            .matmul(&a_t)
            .map_err(|e| NexError::new(format!("lora apply x@A^T: {e}")))?;

        // B has shape [out, r]; B^T has shape [r, out]; (x2 @ A^T) @ B^T = [leading, out].
        let b_t = self
            .b_var
            .as_tensor()
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
            .b_var
            .as_tensor()
            .matmul(self.a_var.as_tensor())
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
                let adapter = LoraAdapter::init(in_f, out_f, cfg, target, layer, &vb, &varmap)?;
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
        let adapter =
            LoraAdapter::init(16, 24, cfg, LoraTarget::QProj, 0, &vb, &varmap).expect("init");
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
        let adapter =
            LoraAdapter::init(8, 8, cfg, LoraTarget::OProj, 3, &vb, &varmap).expect("init");
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

    /// Regression test for the B-stays-zero SFT bug (task #5, 2026-04-19).
    ///
    /// After training, all saved adapter B matrices were exactly zero across
    /// every run of `vigil_train_rapid`, despite AdamW iterating
    /// `bundle.varmap.all_vars()`.  Root hypothesis: `LoraAdapter` holds
    /// `a`/`b` as cloned Tensors from `VarBuilder::get_with_hints`, which are
    /// snapshots — not live views of the underlying `Var`.  Updates to the
    /// Var via optimizer step don't propagate to the adapter's cached tensor,
    /// so `apply()` keeps seeing B=0 forever and delta stays zero.
    ///
    /// This test documents the expected post-fix behavior: after mutating the
    /// underlying `Var` in the VarMap (simulating an AdamW step), a subsequent
    /// `apply()` on the adapter must reflect the new B, not the init value.
    ///
    /// UPDATE 2026-04-19: this test PASSES — live view works fine.  The bug
    /// is in the BACKWARD path, not the forward.  See
    /// `lora_backward_produces_b_gradient` below for the actual failing test.
    #[test]
    fn lora_b_updates_propagate_to_apply() {
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &cpu());
        let cfg = LoraConfig {
            r: 4,
            alpha: 8.0,
            dropout: 0.0,
        };
        let adapter = LoraAdapter::init(8, 8, cfg, LoraTarget::QProj, 0, &vb, &varmap)
            .expect("init adapter");

        // At init, B is zero so apply(x) must be zero.
        let x = Tensor::ones((1, 8), DType::F32, &cpu()).expect("x");
        let init_delta_abs = adapter
            .apply(&x)
            .expect("apply init")
            .abs()
            .expect("abs")
            .sum_all()
            .expect("sum")
            .to_scalar::<f32>()
            .expect("scalar");
        assert!(init_delta_abs < 1e-6, "B starts at zero, delta should be zero");

        // Simulate what AdamW would do: mutate the underlying Var for
        // `lora.0.q_proj.b` to ones.
        let data = varmap.data();
        let data_guard = data.lock().expect("varmap lock");
        let b_var = data_guard
            .get("lora.0.q_proj.b")
            .expect("b var registered");
        let new_b = Tensor::ones((8, 4), DType::F32, &cpu()).expect("new_b");
        b_var.set(&new_b).expect("set new b");
        drop(data_guard);

        // Now apply(x) MUST see the updated B and produce nonzero delta.
        // If LoraAdapter holds a stale Tensor snapshot, this fails → bug confirmed.
        let after_delta_abs = adapter
            .apply(&x)
            .expect("apply after update")
            .abs()
            .expect("abs")
            .sum_all()
            .expect("sum")
            .to_scalar::<f32>()
            .expect("scalar");
        assert!(
            after_delta_abs > 1e-3,
            "after Var::set on B, apply() must reflect new B — got abs-sum {after_delta_abs}."
        );
    }

    /// The true regression test for task #5: does the backward pass actually
    /// produce a nonzero gradient for B?
    ///
    /// In real SFT, after many steps of training, saved B matrices are 100%
    /// zero (verified empirically on vigil-lora-v1 and vigil-lora-farm-v1).
    /// Forward works (test above), so the break is in backward.
    ///
    /// This test does one synthetic forward + backward on a dummy loss, then
    /// checks that the GradStore contains a nonzero gradient for the B var.
    ///
    /// EXPECTED TO FAIL until task #5 is fixed.  Marked `#[ignore]`.
    #[test]
    fn lora_backward_produces_b_gradient() {
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &cpu());
        let cfg = LoraConfig {
            r: 4,
            alpha: 8.0,
            dropout: 0.0,
        };
        let adapter = LoraAdapter::init(8, 8, cfg, LoraTarget::QProj, 0, &vb, &varmap)
            .expect("init adapter");

        // Need B != 0 so delta is nonzero and the gradient path is exercised.
        // (If B=0, ∂loss/∂B is still well-defined via chain rule, but floating
        // point behavior around exact zero can mask real bugs — so start B at
        // a small nonzero value matching what ~1 training step might produce.)
        {
            let data = varmap.data();
            let g = data.lock().expect("varmap lock");
            let b_var = g.get("lora.0.q_proj.b").expect("b var");
            let seed_b = Tensor::new(&[0.01f32; 32], &cpu())
                .expect("seed")
                .reshape((8, 4))
                .expect("reshape");
            b_var.set(&seed_b).expect("seed b");
        }

        let x = Tensor::ones((1, 8), DType::F32, &cpu()).expect("x");
        let delta = adapter.apply(&x).expect("apply");
        // Dummy loss = sum(delta^2) → gradient is straightforwardly nonzero wrt B
        let loss = delta
            .sqr()
            .expect("sqr")
            .sum_all()
            .expect("sum");
        let grads = loss.backward().expect("backward");

        // Find B's var and look up its gradient in the GradStore.
        let data = varmap.data();
        let g = data.lock().expect("lock");
        let b_var = g.get("lora.0.q_proj.b").expect("b var");
        let b_grad = grads.get(b_var.as_tensor()).expect("b grad in store");
        let grad_abs = b_grad
            .abs()
            .expect("abs")
            .sum_all()
            .expect("sum")
            .to_scalar::<f32>()
            .expect("scalar");
        assert!(
            grad_abs > 1e-6,
            "backward() produced zero gradient for B (abs-sum = {grad_abs})."
        );
    }

    /// Full integration regression: mimic the real SFT loop at minimal scale.
    /// Bundle → apply adapter to input → loss → backward → AdamW.step →
    /// verify B changed from its init value.
    ///
    /// If unit tests pass (forward + backward isolated work) but this fails,
    /// the bug is in how SFT composes them — likely one of:
    ///   (a) `bundle.varmap.all_vars()` not returning B vars
    ///   (b) AdamW looking up gradient by a key that doesn't match
    ///   (c) the model wrapping adapter in a Drop-shaped path
    #[test]
    fn lora_sft_cycle_updates_b() {
        use candle_nn::{AdamW, Optimizer, ParamsAdamW};

        let bundle = LoraBundle::attach_canonical(
            1,
            8,
            2,
            2,
            LoraConfig {
                r: 4,
                alpha: 8.0,
                dropout: 0.0,
            },
            DType::F32,
            &cpu(),
        )
        .expect("bundle");

        // Seed B to a small nonzero value so delta is nonzero.
        {
            let data = bundle.varmap.data();
            let g = data.lock().expect("lock");
            for (name, var) in g.iter() {
                if name.ends_with(".b") {
                    let shape = var.as_tensor().dims().to_vec();
                    let n: usize = shape.iter().product();
                    let seed = Tensor::new(vec![0.01f32; n].as_slice(), &cpu())
                        .expect("seed")
                        .reshape(shape)
                        .expect("reshape");
                    var.set(&seed).expect("seed b");
                }
            }
        }

        // Capture B value before training.
        let b_before: f32 = {
            let data = bundle.varmap.data();
            let g = data.lock().expect("lock");
            let v = g.get("lora.0.q_proj.b").expect("b");
            v.as_tensor()
                .abs()
                .expect("abs")
                .sum_all()
                .expect("sum")
                .to_scalar::<f32>()
                .expect("scalar")
        };

        let params = ParamsAdamW {
            lr: 0.1,
            weight_decay: 0.0,
            ..Default::default()
        };
        let mut opt = AdamW::new(bundle.varmap.all_vars(), params).expect("adamw");

        // One step: forward through the adapter, squared-norm loss, backward, step.
        let adapter = bundle
            .find(0, LoraTarget::QProj)
            .expect("adapter present");
        let x = Tensor::ones((1, 8), DType::F32, &cpu()).expect("x");
        let delta = adapter.apply(&x).expect("apply");
        let loss = delta.sqr().expect("sqr").sum_all().expect("sum");
        let grads = loss.backward().expect("backward");
        opt.step(&grads).expect("adam step");

        // B must have moved.
        let b_after: f32 = {
            let data = bundle.varmap.data();
            let g = data.lock().expect("lock");
            let v = g.get("lora.0.q_proj.b").expect("b");
            v.as_tensor()
                .abs()
                .expect("abs")
                .sum_all()
                .expect("sum")
                .to_scalar::<f32>()
                .expect("scalar")
        };
        let delta_change = (b_after - b_before).abs();
        assert!(
            delta_change > 1e-6,
            "B did not change after AdamW step: before={b_before}, after={b_after}. \
             This is the real SFT bug — gradient flows and var is in optimizer's \
             list, but the update doesn't take."
        );
    }

    /// Minimal reproducer for task #5: does autograd propagate gradient from
    /// a Var through `frozen_tensor + var_tensor`?  Real SFT has this pattern
    /// many times over (frozen q_proj output + LoRA delta), and empirically
    /// 0/192 LoRA vars receive gradients in SFT.  If this test fails, the bug
    /// is a candle-specific "tracked op mixed with untracked operand" issue
    /// and the fix is upstream or requires wrapping base paths in a Var.
    ///
    /// EMPIRICAL RESULT 2026-04-19: this test PASSES.  So candle's autograd
    /// does propagate gradient through `frozen + tracked_var`.  The bug is
    /// therefore NOT in the simple add pattern.  See
    /// `qwen2_synthetic_all_var_gets_gradient` next for the narrower repro.
    #[test]
    fn autograd_mixes_frozen_and_var_operands() {
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &cpu());
        let _v = vb
            .get_with_hints((2, 2), "trainable", Init::Const(0.1))
            .expect("register");
        // Retrieve the Var
        let trainable_var = {
            let data = varmap.data();
            let g = data.lock().expect("lock");
            g.get("trainable").expect("var").clone()
        };

        // Frozen tensor created directly (no Var backing, no VarBuilder)
        let frozen = Tensor::ones((2, 2), DType::F32, &cpu()).expect("frozen");
        // Operation mimicking `base_out + delta`:
        let out = (&frozen + trainable_var.as_tensor()).expect("add");
        let loss = out.sqr().expect("sqr").sum_all().expect("sum");
        let grads = loss.backward().expect("backward");

        // Is trainable_var's gradient in the GradStore?
        let present = grads.get(trainable_var.as_tensor()).is_some();
        assert!(
            present,
            "Gradient for Var missing in GradStore when adding frozen + Var."
        );
    }

    /// Narrower task #5 probe: build a tiny Qwen2 model entirely from a
    /// Var-backed VarBuilder (no `from_buffered_safetensors`), attach a LoRA
    /// bundle, run forward → loss → backward, check if ANY LoRA var receives
    /// a gradient.
    ///
    /// If this PASSES: the bug is specifically about
    /// `VarBuilder::from_buffered_safetensors` producing detached tensors
    /// whose subsequent ops don't retain autograd connectivity with
    /// Var-backed adapters.  The fix is to either wrap the base model in a
    /// detached-but-trackable way OR to switch SFT to use `from_varmap` +
    /// loaded weights (set after construction).
    ///
    /// If this FAILS: the bug is structural in the Qwen2 model — some op in
    /// the attention/MLP/norm path breaks autograd even with all-tracked
    /// inputs.
    #[test]
    fn qwen2_synthetic_all_var_gets_gradient() {
        use crate::qwen2_lora::{Config, ModelForCausalLM};

        // Minimal Qwen2 config: 1 layer, tiny dims, still valid shape-wise.
        let cfg = Config {
            hidden_size: 8,
            intermediate_size: 16,
            vocab_size: 32,
            num_hidden_layers: 1,
            num_attention_heads: 2,
            num_key_value_heads: 2,
            max_position_embeddings: 32,
            rms_norm_eps: 1e-6,
            rope_theta: 10000.0,
            sliding_window: 32,
            use_sliding_window: false,
            max_window_layers: 0,
            hidden_act: candle_nn::Activation::Silu,
            tie_word_embeddings: false,
        };

        // ALL vars tracked (not from_buffered_safetensors)
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &cpu());

        let bundle = LoraBundle::attach_canonical(
            cfg.num_hidden_layers,
            cfg.hidden_size,
            cfg.num_attention_heads,
            cfg.num_key_value_heads,
            LoraConfig {
                r: 4,
                alpha: 8.0,
                dropout: 0.0,
            },
            DType::F32,
            &cpu(),
        )
        .expect("bundle");

        // Seed LoRA B to small nonzero so delta isn't literally zero
        {
            let data = bundle.varmap.data();
            let g = data.lock().expect("lock");
            for (name, var) in g.iter() {
                if name.ends_with(".b") {
                    let shape = var.as_tensor().dims().to_vec();
                    let n: usize = shape.iter().product();
                    let seed = Tensor::new(vec![0.01f32; n].as_slice(), &cpu())
                        .expect("seed")
                        .reshape(shape)
                        .expect("reshape");
                    var.set(&seed).expect("set");
                }
            }
        }

        let mut model = ModelForCausalLM::new_with_lora(&cfg, vb, Some(&bundle))
            .expect("build model");

        // Dummy input: batch=1, seq_len=4, vocab_id range
        let input =
            Tensor::new(&[[0u32, 1, 2, 3]], &cpu()).expect("input");
        model.clear_kv_cache();
        let logits = model.forward_all(&input, 0).expect("forward");
        let loss = logits
            .to_dtype(DType::F32)
            .expect("f32")
            .sqr()
            .expect("sqr")
            .sum_all()
            .expect("sum");

        let grads = loss.backward().expect("backward");

        // Count LoRA vars with gradients
        let mut lora_with_grad = 0usize;
        let mut lora_total = 0usize;
        let data = bundle.varmap.data();
        let g = data.lock().expect("lock");
        for (_name, var) in g.iter() {
            lora_total += 1;
            if grads.get(var.as_tensor()).is_some() {
                lora_with_grad += 1;
            }
        }
        assert!(
            lora_with_grad > 0,
            "0/{lora_total} LoRA vars have gradients in an all-Var-backed \
             synthetic Qwen2. Bug is structural in the model — NOT about \
             from_buffered_safetensors. Look for an op (contiguous, softmax, \
             rotary, etc.) that severs autograd inside the attention forward."
        );
    }

    /// Bisect task #5: does the bug exist at the simplest possible level —
    /// just Embedding + Linear + LoRA-delta composition — or only in the
    /// full attention/rotary/softmax path?
    ///
    /// If this PASSES: bug is specifically in attention/MLP/norm/rotary.
    /// If this FAILS: bug is at the Var-backed VarBuilder + adapter-apply
    /// composition level (much broader).
    #[test]
    fn simple_embed_linear_lora_grad_flows() {
        use candle_nn::Module;

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &cpu());

        // Embedding: vocab 32 → hidden 8
        let embed = candle_nn::embedding(32, 8, vb.pp("embed")).expect("embed");
        // Linear: 8 → 8
        let lin = candle_nn::linear_no_bias(8, 8, vb.pp("lin")).expect("linear");

        // LoRA bundle attached to a single "layer 0, q_proj" for 8-dim
        let bundle = LoraBundle::attach_canonical(
            1,
            8,
            2,
            2,
            LoraConfig {
                r: 4,
                alpha: 8.0,
                dropout: 0.0,
            },
            DType::F32,
            &cpu(),
        )
        .expect("bundle");

        // Seed B to nonzero
        {
            let data = bundle.varmap.data();
            let g = data.lock().expect("lock");
            for (name, var) in g.iter() {
                if name.ends_with(".b") {
                    let shape = var.as_tensor().dims().to_vec();
                    let n: usize = shape.iter().product();
                    let seed = Tensor::new(vec![0.01f32; n].as_slice(), &cpu())
                        .expect("seed")
                        .reshape(shape)
                        .expect("reshape");
                    var.set(&seed).expect("set");
                }
            }
        }
        let adapter = bundle.find(0, LoraTarget::QProj).expect("adapter");

        // Forward: input_ids → embed → linear → + LoRA delta → loss
        let input = Tensor::new(&[[0u32, 1, 2, 3]], &cpu()).expect("input");
        let x = embed.forward(&input).expect("embed forward"); // [1, 4, 8]
        let x2d = x.reshape((4, 8)).expect("reshape 2d");
        let base_out = lin.forward(&x2d).expect("linear forward"); // [4, 8]
        let delta = adapter.apply(&x2d).expect("apply"); // [4, 8]
        let out = (&base_out + &delta).expect("add");
        let loss = out.sqr().expect("sqr").sum_all().expect("sum");
        let grads = loss.backward().expect("backward");

        let mut lora_with_grad = 0usize;
        let mut lora_total = 0usize;
        let data = bundle.varmap.data();
        let g = data.lock().expect("lock");
        for (_name, var) in g.iter() {
            lora_total += 1;
            if grads.get(var.as_tensor()).is_some() {
                lora_with_grad += 1;
            }
        }
        assert!(
            lora_with_grad > 0,
            "0/{lora_total} LoRA vars have gradients in simple embed+linear+LoRA. \
             Bug is broader than attention — it's in adapter.apply composition \
             with upstream Var-backed tensors from a DIFFERENT varmap (the model's)."
        );
    }

    /// Second bisect: is the bug specific to multi-head attention, or does it
    /// show up even with a single head (no multi-head reshape/transpose)?
    /// Uses the same synthetic Qwen2 test but with num_attention_heads=1.
    #[test]
    fn qwen2_synthetic_single_head() {
        use crate::qwen2_lora::{Config, ModelForCausalLM};

        let cfg = Config {
            hidden_size: 8,
            intermediate_size: 16,
            vocab_size: 32,
            num_hidden_layers: 1,
            num_attention_heads: 1,
            num_key_value_heads: 1,
            max_position_embeddings: 32,
            rms_norm_eps: 1e-6,
            rope_theta: 10000.0,
            sliding_window: 32,
            use_sliding_window: false,
            max_window_layers: 0,
            hidden_act: candle_nn::Activation::Silu,
            tie_word_embeddings: false,
        };

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &cpu());

        let bundle = LoraBundle::attach_canonical(
            cfg.num_hidden_layers,
            cfg.hidden_size,
            cfg.num_attention_heads,
            cfg.num_key_value_heads,
            LoraConfig {
                r: 4,
                alpha: 8.0,
                dropout: 0.0,
            },
            DType::F32,
            &cpu(),
        )
        .expect("bundle");

        // Seed B nonzero
        {
            let data = bundle.varmap.data();
            let g = data.lock().expect("lock");
            for (name, var) in g.iter() {
                if name.ends_with(".b") {
                    let shape = var.as_tensor().dims().to_vec();
                    let n: usize = shape.iter().product();
                    let seed = Tensor::new(vec![0.01f32; n].as_slice(), &cpu())
                        .expect("seed")
                        .reshape(shape)
                        .expect("reshape");
                    var.set(&seed).expect("set");
                }
            }
        }

        let mut model = ModelForCausalLM::new_with_lora(&cfg, vb, Some(&bundle))
            .expect("build model");
        let input = Tensor::new(&[[0u32, 1, 2, 3]], &cpu()).expect("input");
        model.clear_kv_cache();
        let logits = model.forward_all(&input, 0).expect("forward");
        let loss = logits
            .to_dtype(DType::F32)
            .expect("f32")
            .sqr()
            .expect("sqr")
            .sum_all()
            .expect("sum");
        let grads = loss.backward().expect("backward");

        let mut lora_with_grad = 0usize;
        let mut lora_total = 0usize;
        let data = bundle.varmap.data();
        let g = data.lock().expect("lock");
        for (_name, var) in g.iter() {
            lora_total += 1;
            if grads.get(var.as_tensor()).is_some() {
                lora_with_grad += 1;
            }
        }
        assert!(
            lora_with_grad > 0,
            "0/{lora_total} LoRA vars have gradients in SINGLE-HEAD Qwen2. \
             Bug is NOT multi-head-specific — it's in attention structurally \
             (rotary, softmax, KV cache, residual, or layernorm)."
        );
    }

    /// Third bisect: skip the lm_head.  Call `Model::forward` directly (not
    /// `ModelForCausalLM::forward_all`), compute loss on the final hidden
    /// states instead of logits.  If this PASSES, the bug is in lm_head.
    /// If it FAILS, the bug is in the transformer layers.
    #[test]
    fn qwen2_synthetic_no_lm_head() {
        use crate::qwen2_lora::{Config, Model};

        let cfg = Config {
            hidden_size: 8,
            intermediate_size: 16,
            vocab_size: 32,
            num_hidden_layers: 1,
            num_attention_heads: 1,
            num_key_value_heads: 1,
            max_position_embeddings: 32,
            rms_norm_eps: 1e-6,
            rope_theta: 10000.0,
            sliding_window: 32,
            use_sliding_window: false,
            max_window_layers: 0,
            hidden_act: candle_nn::Activation::Silu,
            tie_word_embeddings: false,
        };

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &cpu());

        let bundle = LoraBundle::attach_canonical(
            cfg.num_hidden_layers,
            cfg.hidden_size,
            cfg.num_attention_heads,
            cfg.num_key_value_heads,
            LoraConfig {
                r: 4,
                alpha: 8.0,
                dropout: 0.0,
            },
            DType::F32,
            &cpu(),
        )
        .expect("bundle");

        {
            let data = bundle.varmap.data();
            let g = data.lock().expect("lock");
            for (name, var) in g.iter() {
                if name.ends_with(".b") {
                    let shape = var.as_tensor().dims().to_vec();
                    let n: usize = shape.iter().product();
                    let seed = Tensor::new(vec![0.01f32; n].as_slice(), &cpu())
                        .expect("seed")
                        .reshape(shape)
                        .expect("reshape");
                    var.set(&seed).expect("set");
                }
            }
        }

        let mut model = Model::new_with_lora(&cfg, vb, Some(&bundle)).expect("build");
        let input = Tensor::new(&[[0u32, 1, 2, 3]], &cpu()).expect("input");
        model.clear_kv_cache();
        let hidden = model.forward(&input, 0, None).expect("forward");
        let loss = hidden
            .to_dtype(DType::F32)
            .expect("f32")
            .sqr()
            .expect("sqr")
            .sum_all()
            .expect("sum");
        let grads = loss.backward().expect("backward");

        let mut lora_with_grad = 0usize;
        let mut lora_total = 0usize;
        let data = bundle.varmap.data();
        let g = data.lock().expect("lock");
        for (_name, var) in g.iter() {
            lora_total += 1;
            if grads.get(var.as_tensor()).is_some() {
                lora_with_grad += 1;
            }
        }
        assert!(
            lora_with_grad > 0,
            "0/{lora_total} LoRA vars have gradients when calling Model::forward \
             directly (no lm_head). Bug is in the transformer layers, not lm_head."
        );
    }

    /// Fourth bisect: build Qwen2 with `bundle=None` (no LoRA in attention),
    /// then compute the LoRA delta externally by calling `adapter.apply(xs)`
    /// on the embedded input, and add it to the final model output.
    ///
    /// If this PASSES: the bug is specifically in how adapter.apply
    /// composes INSIDE the attention forward — the transformer `forward`
    /// itself is fine.  Fix direction: move LoRA out of attention, apply
    /// externally.
    ///
    /// If this FAILS: running the transformer forward somehow taints
    /// downstream gradient flow regardless of where LoRA is added.
    #[test]
    fn qwen2_external_lora_grad_flows() {
        use crate::qwen2_lora::{Config, Model};

        let cfg = Config {
            hidden_size: 8,
            intermediate_size: 16,
            vocab_size: 32,
            num_hidden_layers: 1,
            num_attention_heads: 1,
            num_key_value_heads: 1,
            max_position_embeddings: 32,
            rms_norm_eps: 1e-6,
            rope_theta: 10000.0,
            sliding_window: 32,
            use_sliding_window: false,
            max_window_layers: 0,
            hidden_act: candle_nn::Activation::Silu,
            tie_word_embeddings: false,
        };

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &cpu());
        let mut model = Model::new_with_lora(&cfg, vb, None).expect("build no-lora model");

        // External LoRA bundle — not plumbed into model
        let bundle = LoraBundle::attach_canonical(
            1,
            cfg.hidden_size,
            cfg.num_attention_heads,
            cfg.num_key_value_heads,
            LoraConfig {
                r: 4,
                alpha: 8.0,
                dropout: 0.0,
            },
            DType::F32,
            &cpu(),
        )
        .expect("bundle");
        {
            let data = bundle.varmap.data();
            let g = data.lock().expect("lock");
            for (name, var) in g.iter() {
                if name.ends_with(".b") {
                    let shape = var.as_tensor().dims().to_vec();
                    let n: usize = shape.iter().product();
                    let seed = Tensor::new(vec![0.01f32; n].as_slice(), &cpu())
                        .expect("seed")
                        .reshape(shape)
                        .expect("reshape");
                    var.set(&seed).expect("set");
                }
            }
        }
        let adapter = bundle.find(0, LoraTarget::QProj).expect("adapter");

        let input = Tensor::new(&[[0u32, 1, 2, 3]], &cpu()).expect("input");
        model.clear_kv_cache();
        let hidden = model.forward(&input, 0, None).expect("forward"); // [1, 4, 8]

        // Add LoRA delta externally: flatten, apply, reshape
        let h2d = hidden.reshape((4, 8)).expect("h2d");
        let delta = adapter.apply(&h2d).expect("apply"); // [4, 8]
        let out = (&h2d + &delta).expect("add");

        let loss = out.sqr().expect("sqr").sum_all().expect("sum");
        let grads = loss.backward().expect("backward");

        let mut lora_with_grad = 0usize;
        let mut lora_total = 0usize;
        let data = bundle.varmap.data();
        let g = data.lock().expect("lock");
        for (_name, var) in g.iter() {
            lora_total += 1;
            if grads.get(var.as_tensor()).is_some() {
                lora_with_grad += 1;
            }
        }
        assert!(
            lora_with_grad > 0,
            "0/{lora_total} external LoRA vars have gradients after Qwen2 forward + \
             external adapter apply. Bug is broader than \"lora inside attention\" — \
             running the Qwen2 transformer forward taints downstream gradient flow."
        );
    }
}
