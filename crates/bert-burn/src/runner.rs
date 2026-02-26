use crate::checkpoint::{CheckpointManager, CheckpointMetadata};
use crate::config::RunConfig;
use crate::{data, model, training};
use burn::backend::Autodiff;
use burn::backend::ndarray::NdArray;
use burn::grad_clipping::GradientClippingConfig;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::tensor::backend::AutodiffBackend;
use burn::tensor::{Bool, Int, Tensor, TensorData};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Instant;

pub type MyBackend = NdArray;
pub type Backend = Autodiff<MyBackend>;

#[derive(Debug, Clone)]
pub struct TrainingOutcome {
    pub final_loss: f32,
    pub final_accuracy: f32,
    pub total_steps: usize,
    pub elapsed_secs: f32,
    pub last_checkpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStatus {
    pub run_id: String,
    pub phase: String,
    pub epoch: usize,
    pub batch: usize,
    pub global_step: usize,
    pub loss: f32,
    pub accuracy: f32,
    pub elapsed_secs: f32,
}

fn write_live_status(
    meta_dir: &str,
    status: &LiveStatus,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(meta_dir)?;
    let path = format!("{}/live_status.json", meta_dir);
    fs::write(path, serde_json::to_string_pretty(status)?)?;
    Ok(())
}

/// Convert Vec<Vec<i64>> batch to Burn tensor
fn batch_to_tensor<B: AutodiffBackend>(
    input_ids: &[Vec<i64>],
    device: &B::Device,
) -> Tensor<B, 2, Int> {
    assert!(
        !input_ids.is_empty(),
        "batch_to_tensor: input_ids must not be empty"
    );
    let batch_size = input_ids.len();
    let seq_len = input_ids[0].len();
    let flat: Vec<i64> = input_ids
        .iter()
        .flat_map(|row| row.iter().copied())
        .collect();
    let data = TensorData::from(flat.as_slice());
    let t: Tensor<B, 1, Int> = Tensor::from_data(data, device);
    t.reshape([batch_size, seq_len])
}

/// Convert attention mask (1.0=real, 0.0=pad) to burn padding mask (true=pad).
fn mask_to_pad_tensor<B: AutodiffBackend>(
    attention_mask: &[Vec<f32>],
    device: &B::Device,
) -> Tensor<B, 2, Bool> {
    assert!(
        !attention_mask.is_empty(),
        "mask_to_pad_tensor: attention_mask must not be empty"
    );
    let batch_size = attention_mask.len();
    let seq_len = attention_mask[0].len();
    let flat: Vec<f32> = attention_mask
        .iter()
        .flat_map(|row| row.iter().copied())
        .collect();
    let data = TensorData::from(flat.as_slice());
    let float_tensor: Tensor<B, 1> = Tensor::from_data(data, device);
    float_tensor.reshape([batch_size, seq_len]).equal_elem(0.0) // true = padding position (masked from attention)
}

pub fn run_training(
    config: &RunConfig,
    checkpoint_dir: &str,
    meta_dir: &str,
    run_id: &str,
) -> Result<TrainingOutcome, Box<dyn std::error::Error>> {
    let run_start = Instant::now();

    println!("BERT DNA-Assembler Training");
    println!("===========================\n");

    println!("Model Config:");
    println!("  - Vocab: {}", config.vocab_size);
    println!("  - Seq Length: {}", config.seq_length);
    println!("  - Hidden Dim: {}", config.hidden_dim);
    println!("  - Layers: {}", config.num_layers);
    println!("  - Heads: {}", config.num_heads);

    let device = <Backend as burn::tensor::backend::Backend>::Device::default();

    let mut model = model::BertModel::<Backend>::new(
        config.vocab_size,
        config.seq_length,
        config.hidden_dim,
        config.num_layers,
        config.num_heads,
        config.ff_dim,
    );
    println!("Model initialized\n");

    // Adam optimizer with gradient clipping
    let mut optim = AdamConfig::new()
        .with_grad_clipping(Some(GradientClippingConfig::Norm(config.max_grad_norm)))
        .init();

    let train_cfg = training::TrainingConfig {
        num_epochs: config.num_epochs,
        learning_rate: config.learning_rate,
        warmup_steps: config.warmup_steps,
        log_interval: config.log_interval,
    };

    let checkpoint_mgr = CheckpointManager::new(checkpoint_dir);
    println!("Checkpoint dir: {}\n", checkpoint_dir);

    println!("Loading DNA dataset...");
    let dataset = data::TextDataset::new(config.batch_size, config.seq_length);
    let num_batches = dataset.num_batches();
    println!(
        "Dataset loaded | {} sequences | {} batches\n",
        dataset.num_sequences(),
        num_batches
    );

    let start_epoch = if let Ok(Some(latest)) = checkpoint_mgr.latest_checkpoint() {
        match checkpoint_mgr.load_metadata(&latest) {
            Ok(metadata) => {
                match checkpoint_mgr.load_model::<Backend, _>(model.clone(), &latest, &device) {
                    Ok(loaded) => {
                        model = loaded;
                        println!(
                            "Resumed model from checkpoint: epoch {}, step {}\n",
                            metadata.epoch, metadata.global_step
                        );
                    }
                    Err(e) => {
                        println!("Could not load model weights ({}), starting fresh\n", e);
                    }
                }
                metadata.epoch
            }
            Err(_) => 0,
        }
    } else {
        0
    };

    println!("Starting Training Loop");
    println!("----------------------");

    let mut state = training::TrainingState::new();
    let mut last_checkpoint = None;

    for epoch in start_epoch..train_cfg.num_epochs {
        state.reset_batch_stats();
        let mut best_loss = f32::MAX;

        for batch_idx in 0..num_batches.min(config.max_batches_per_epoch) {
            // Get real batch from dataset
            let batch = match dataset.get_batch(batch_idx) {
                Some(b) => b,
                None => continue,
            };

            // Convert to tensors
            let input_ids = batch_to_tensor::<Backend>(&batch.input_ids, &device);
            let pad_mask = mask_to_pad_tensor::<Backend>(&batch.attention_mask, &device);

            // Forward pass (with padding-aware attention)
            let logits = model.forward(input_ids, Some(pad_mask), true);

            // Compute real MLM loss
            let (loss, accuracy) = training::compute_real_mlm_loss(
                logits,
                &batch.masked_positions,
                &batch.masked_ids,
                config.vocab_size,
            );

            // Extract scalar loss for logging (before backward)
            let loss_scalar: f32 = loss
                .clone()
                .into_data()
                .to_vec::<f32>()
                .unwrap_or_default()
                .first()
                .copied()
                .unwrap_or(0.0);

            // Backward pass + optimizer step
            let grads = loss.backward();
            let grads = GradientsParams::from_grads(grads, &model);
            let lr = state.current_lr(&train_cfg) as f64;
            model = optim.step(lr, model, grads);

            state.update(loss_scalar, accuracy);

            if loss_scalar < best_loss {
                best_loss = loss_scalar;
            }

            if (batch_idx + 1) % train_cfg.log_interval == 0 {
                let metrics = state.get_metrics(
                    epoch + 1,
                    batch_idx + 1,
                    &train_cfg,
                    config.batch_size,
                    config.seq_length,
                );
                metrics.print();
                let _ = write_live_status(
                    meta_dir,
                    &LiveStatus {
                        run_id: run_id.to_string(),
                        phase: "training".to_string(),
                        epoch: epoch + 1,
                        batch: batch_idx + 1,
                        global_step: state.global_step,
                        loss: metrics.loss,
                        accuracy: metrics.accuracy,
                        elapsed_secs: run_start.elapsed().as_secs_f32(),
                    },
                );
            }
        }

        let checkpoint_name = format!("epoch_{:03}_step_{:05}", epoch + 1, state.global_step);
        let metadata = CheckpointMetadata::new(
            epoch + 1,
            0,
            state.global_step,
            state.total_loss / state.batch_count.max(1) as f32,
            state.total_accuracy / state.batch_count.max(1) as f32,
            train_cfg.learning_rate,
        );

        if let Err(e) = checkpoint_mgr.save_metadata(&metadata, &checkpoint_name) {
            eprintln!("Failed to save checkpoint metadata: {}", e);
        }
        if let Err(e) = checkpoint_mgr.save_model::<Backend, _>(model.clone(), &checkpoint_name) {
            eprintln!("Failed to save model weights: {}", e);
        } else {
            last_checkpoint = Some(checkpoint_name);
        }

        println!(
            "  Epoch {} completed | Best Loss: {:.4} | Checkpoint saved",
            epoch + 1,
            best_loss
        );
        println!();
    }

    let final_loss = state.total_loss / state.batch_count.max(1) as f32;
    let final_accuracy = state.total_accuracy / state.batch_count.max(1) as f32;

    println!("Training complete!");
    println!("\nFinal Metrics:");
    println!("  - Total steps: {}", state.global_step);
    println!("  - Final loss: {:.4}", final_loss);
    println!("  - Final accuracy: {:.4}", final_accuracy);

    let _ = write_live_status(
        meta_dir,
        &LiveStatus {
            run_id: run_id.to_string(),
            phase: "completed".to_string(),
            epoch: train_cfg.num_epochs,
            batch: state.batch_count,
            global_step: state.global_step,
            loss: final_loss,
            accuracy: final_accuracy,
            elapsed_secs: run_start.elapsed().as_secs_f32(),
        },
    );

    Ok(TrainingOutcome {
        final_loss,
        final_accuracy,
        total_steps: state.global_step,
        elapsed_secs: run_start.elapsed().as_secs_f32(),
        last_checkpoint,
    })
}
