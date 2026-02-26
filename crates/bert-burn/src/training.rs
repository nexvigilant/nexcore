use burn::nn::loss::{CrossEntropyLoss, CrossEntropyLossConfig};
use burn::tensor::backend::AutodiffBackend;
use burn::tensor::{Int, Tensor};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TrainingMetrics {
    pub epoch: usize,
    pub batch: usize,
    pub loss: f32,
    pub accuracy: f32,
    pub learning_rate: f32,
    pub throughput: f32,
    pub elapsed_secs: f32,
}

impl TrainingMetrics {
    pub fn print(&self) {
        println!(
            "[{:03}] Batch {:4} | Loss: {:.4} | Acc: {:.4} | LR: {:.2e} | {:.0} tok/s | {:.1}s",
            self.epoch,
            self.batch,
            self.loss,
            self.accuracy,
            self.learning_rate,
            self.throughput,
            self.elapsed_secs
        );
    }
}

pub struct TrainingConfig {
    pub num_epochs: usize,
    pub learning_rate: f32,
    pub warmup_steps: usize,
    pub log_interval: usize,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            num_epochs: 3,
            learning_rate: 1e-4,
            warmup_steps: 100,
            log_interval: 5,
        }
    }
}

pub struct TrainingState {
    pub global_step: usize,
    pub total_loss: f32,
    pub total_accuracy: f32,
    pub batch_count: usize,
    pub start_time: Instant,
}

impl TrainingState {
    pub fn new() -> Self {
        Self {
            global_step: 0,
            total_loss: 0.0,
            total_accuracy: 0.0,
            batch_count: 0,
            start_time: Instant::now(),
        }
    }

    pub fn update(&mut self, loss: f32, accuracy: f32) {
        self.total_loss += loss;
        self.total_accuracy += accuracy;
        self.batch_count += 1;
        self.global_step += 1;
    }

    pub fn get_metrics(
        &self,
        epoch: usize,
        batch: usize,
        config: &TrainingConfig,
        batch_size: usize,
        seq_length: usize,
    ) -> TrainingMetrics {
        let avg_loss = if self.batch_count > 0 {
            self.total_loss / self.batch_count as f32
        } else {
            0.0
        };
        let avg_accuracy = if self.batch_count > 0 {
            self.total_accuracy / self.batch_count as f32
        } else {
            0.0
        };
        let elapsed = self.start_time.elapsed().as_secs_f32();

        let current_lr = if self.global_step < config.warmup_steps {
            config.learning_rate * self.global_step as f32 / config.warmup_steps as f32
        } else {
            config.learning_rate
        };

        let throughput = if elapsed > 0.0 {
            (self.batch_count as f32 * batch_size as f32 * seq_length as f32) / elapsed
        } else {
            0.0
        };

        TrainingMetrics {
            epoch,
            batch,
            loss: avg_loss,
            accuracy: avg_accuracy,
            learning_rate: current_lr,
            throughput,
            elapsed_secs: elapsed,
        }
    }

    pub fn current_lr(&self, config: &TrainingConfig) -> f32 {
        if self.global_step < config.warmup_steps {
            config.learning_rate * self.global_step as f32 / config.warmup_steps.max(1) as f32
        } else {
            config.learning_rate
        }
    }

    pub fn reset_batch_stats(&mut self) {
        self.total_loss = 0.0;
        self.total_accuracy = 0.0;
        self.batch_count = 0;
    }
}

/// Real MLM loss: cross-entropy at masked positions
/// Returns (loss_tensor, accuracy_f32)
pub fn compute_real_mlm_loss<B: AutodiffBackend>(
    logits: Tensor<B, 3>,
    masked_positions: &[Vec<i64>],
    masked_ids: &[Vec<i64>],
    vocab_size: usize,
) -> (Tensor<B, 1>, f32) {
    let device = logits.device();
    let mut all_logits: Vec<Tensor<B, 2>> = Vec::new();
    let mut all_targets: Vec<i64> = Vec::new();
    let mut total = 0usize;

    for (batch_i, (positions, targets)) in
        masked_positions.iter().zip(masked_ids.iter()).enumerate()
    {
        for (idx, &pos) in positions.iter().enumerate() {
            if idx >= targets.len() {
                break;
            }
            let Some(pos) = usize::try_from(pos).ok() else {
                continue;
            };

            let logit_vec = logits
                .clone()
                .slice([batch_i..batch_i + 1, pos..pos + 1, 0..vocab_size])
                .reshape([1, vocab_size]);
            all_logits.push(logit_vec);
            all_targets.push(targets[idx]);
            total += 1;
        }
    }

    if all_logits.is_empty() || all_targets.is_empty() {
        let zero = Tensor::<B, 1>::zeros([1], &device);
        return (zero, 0.0);
    }

    let gathered = Tensor::cat(all_logits, 0); // [N, vocab_size]

    // Compute accuracy
    let pred = gathered.clone().argmax(1).reshape([total]);
    let target_data: Vec<i64> = all_targets;
    let target_tensor: Tensor<B, 1, Int> = Tensor::from_data(
        burn::tensor::TensorData::from(target_data.as_slice()),
        &device,
    );

    let accuracy_tensor = pred.equal(target_tensor.clone());
    let num_correct: f32 = accuracy_tensor
        .into_data()
        .to_vec::<bool>()
        .unwrap_or_default()
        .iter()
        .filter(|&&b| b)
        .count() as f32;

    let loss_fn: CrossEntropyLoss<B> = CrossEntropyLossConfig::new().init(&device);
    let loss = loss_fn.forward(gathered, target_tensor);

    let accuracy = if total > 0 {
        num_correct / total as f32
    } else {
        0.0
    };

    (loss.reshape([1]), accuracy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_state_update() {
        let mut state = TrainingState::new();
        state.update(2.0, 0.5);
        state.update(1.0, 0.7);
        assert_eq!(state.global_step, 2);
        assert_eq!(state.batch_count, 2);
        assert!((state.total_loss - 3.0).abs() < 1e-6);
        assert!((state.total_accuracy - 1.2).abs() < 1e-6);
    }

    #[test]
    fn test_training_state_reset() {
        let mut state = TrainingState::new();
        state.update(2.0, 0.5);
        state.update(1.0, 0.7);
        state.reset_batch_stats();
        assert_eq!(state.batch_count, 0);
        assert_eq!(state.total_loss, 0.0);
        assert_eq!(state.total_accuracy, 0.0);
        // global_step should NOT reset
        assert_eq!(state.global_step, 2);
    }

    #[test]
    fn test_lr_warmup() {
        let cfg = TrainingConfig {
            num_epochs: 1,
            learning_rate: 1e-4,
            warmup_steps: 10,
            log_interval: 1,
        };
        let mut state = TrainingState::new();
        // Step 0: lr = 0
        assert_eq!(state.current_lr(&cfg), 0.0);
        // Step 5: lr = 5e-5
        for _ in 0..5 {
            state.update(1.0, 0.0);
        }
        assert!((state.current_lr(&cfg) - 5e-5).abs() < 1e-8);
        // Step 10+: full lr
        for _ in 0..5 {
            state.update(1.0, 0.0);
        }
        assert!((state.current_lr(&cfg) - 1e-4).abs() < 1e-8);
    }

    #[test]
    fn test_metrics_computation() {
        let cfg = TrainingConfig {
            num_epochs: 1,
            learning_rate: 1e-4,
            warmup_steps: 100,
            log_interval: 1,
        };
        let mut state = TrainingState::new();
        state.update(2.5, 0.3);
        state.update(1.5, 0.7);
        let metrics = state.get_metrics(1, 2, &cfg, 4, 64);
        assert_eq!(metrics.epoch, 1);
        assert_eq!(metrics.batch, 2);
        assert!((metrics.loss - 2.0).abs() < 1e-6);
        assert!((metrics.accuracy - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_mlm_loss_empty_masks() {
        use burn::backend::Autodiff;
        use burn::backend::ndarray::NdArray;
        type B = Autodiff<NdArray>;
        let device = <B as burn::tensor::backend::Backend>::Device::default();
        let logits = Tensor::<B, 3>::zeros([2, 8, 9], &device);
        let (loss, acc) = compute_real_mlm_loss(logits, &[vec![], vec![]], &[vec![], vec![]], 9);
        let loss_val: f32 = loss
            .into_data()
            .to_vec::<f32>()
            .unwrap_or_default()
            .first()
            .copied()
            .unwrap_or(f32::NAN);
        assert_eq!(loss_val, 0.0);
        assert_eq!(acc, 0.0);
    }

    /// Verify compute_real_mlm_loss actually computes loss and accuracy
    /// when masked positions are provided. The empty-masks test only exercises
    /// the early-return path; this exercises the real computation path.
    #[test]
    fn test_mlm_loss_with_real_positions() {
        use burn::backend::Autodiff;
        use burn::backend::ndarray::NdArray;
        use burn::tensor::TensorData;
        type B = Autodiff<NdArray>;
        let device = <B as burn::tensor::backend::Backend>::Device::default();

        // Logits: 1 batch, 8 positions, vocab_size=9
        // Set logit at position 3 to strongly predict class 5 (A)
        let mut logit_data = vec![0.0_f32; 1 * 8 * 9];
        // Position 3, class 5 (A): large logit so argmax selects class 5
        logit_data[3 * 9 + 5] = 10.0;

        let logits = Tensor::<B, 1>::from_data(TensorData::from(logit_data.as_slice()), &device)
            .reshape([1, 8, 9]);

        // Mask position 3, true label = 5 (A) — model should get this right
        let masked_positions = vec![vec![3_i64]];
        let masked_ids = vec![vec![5_i64]];

        let (loss, acc) = compute_real_mlm_loss(logits, &masked_positions, &masked_ids, 9);

        let loss_val: f32 = loss
            .into_data()
            .to_vec::<f32>()
            .unwrap_or_default()
            .first()
            .copied()
            .unwrap_or(f32::NAN);

        // Loss must be finite and positive
        assert!(
            loss_val.is_finite(),
            "loss must be finite, got {}",
            loss_val
        );
        assert!(
            loss_val >= 0.0,
            "loss must be non-negative, got {}",
            loss_val
        );
        // With a dominant logit pointing to the correct class, accuracy should be 1.0
        assert!(
            (acc - 1.0).abs() < 1e-6,
            "accuracy should be 1.0 when logits strongly favour the correct class, got {}",
            acc
        );
        // Cross-entropy loss with a near-certain correct prediction should be low
        assert!(
            loss_val < 0.01,
            "loss should be near-zero for a confident correct prediction, got {}",
            loss_val
        );
    }
}
