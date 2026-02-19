use crate::config::RunConfig;
use crate::runner::TrainingOutcome;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveWeights {
    pub accuracy: f32,
    pub loss: f32,
    pub throughput: f32,
}

impl Default for ObjectiveWeights {
    fn default() -> Self {
        Self {
            accuracy: 0.5,
            loss: 0.3,
            throughput: 0.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    pub timestamp_unix: u64,
    pub checkpoint_dir: String,
    pub score: f32,
    pub final_loss: f32,
    pub final_accuracy: f32,
    pub total_steps: usize,
    pub elapsed_secs: f32,
    pub config: RunConfig,
}

pub fn score_run(outcome: &TrainingOutcome, cfg: &RunConfig, weights: &ObjectiveWeights) -> f32 {
    let acc_component = outcome.final_accuracy * 100.0;
    let loss_component = (2.0 / (0.1 + outcome.final_loss.max(0.0))) * 50.0; // Higher reward for near-zero loss

    let tokens_total = outcome.total_steps as f32 * cfg.batch_size as f32 * cfg.seq_length as f32;
    let tps = if outcome.elapsed_secs > 0.0 {
        tokens_total / outcome.elapsed_secs
    } else {
        0.0
    };
    let throughput_component = (tps / 5_000.0) * 100.0; // Lowered denominator to reward high speed more aggressively

    (acc_component * weights.accuracy)
        + (loss_component * weights.loss)
        + (throughput_component * weights.throughput)
}

pub fn persist_run(
    meta_dir: &str,
    run_id: &str,
    checkpoint_dir: &str,
    cfg: &RunConfig,
    outcome: &TrainingOutcome,
    score: f32,
) -> Result<RunRecord, Box<dyn std::error::Error>> {
    fs::create_dir_all(meta_dir)?;
    let runs_dir = format!("{}/runs", meta_dir);
    fs::create_dir_all(&runs_dir)?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let record = RunRecord {
        run_id: run_id.to_string(),
        timestamp_unix: now,
        checkpoint_dir: checkpoint_dir.to_string(),
        score,
        final_loss: outcome.final_loss,
        final_accuracy: outcome.final_accuracy,
        total_steps: outcome.total_steps,
        elapsed_secs: outcome.elapsed_secs,
        config: cfg.clone(),
    };

    let run_path = format!("{}/{}.json", runs_dir, run_id);
    fs::write(&run_path, serde_json::to_string_pretty(&record)?)?;

    update_leaderboard(meta_dir, &record)?;
    Ok(record)
}

pub fn load_leaderboard(meta_dir: &str) -> Result<Vec<RunRecord>, Box<dyn std::error::Error>> {
    let board_path = format!("{}/leaderboard.json", meta_dir);
    if !Path::new(&board_path).exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(board_path)?;
    let records: Vec<RunRecord> = serde_json::from_str(&raw)?;
    Ok(records)
}

fn update_leaderboard(
    meta_dir: &str,
    new_record: &RunRecord,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut records = load_leaderboard(meta_dir)?;
    records.push(new_record.clone());
    records.sort_by(|a, b| b.score.total_cmp(&a.score));

    let board_path = format!("{}/leaderboard.json", meta_dir);
    fs::write(&board_path, serde_json::to_string_pretty(&records)?)?;
    Ok(())
}

pub fn print_leaderboard(meta_dir: &str, top_n: usize) -> Result<(), Box<dyn std::error::Error>> {
    let records = load_leaderboard(meta_dir)?;
    if records.is_empty() {
        println!("No runs recorded yet in {}", meta_dir);
        return Ok(());
    }

    println!("🏆 Leaderboard (top {})", top_n);
    for (idx, r) in records.iter().take(top_n).enumerate() {
        println!(
            "{:>2}. {} | score {:.2} | acc {:.4} | loss {:.4} | {}",
            idx + 1,
            r.run_id,
            r.score,
            r.final_accuracy,
            r.final_loss,
            r.checkpoint_dir
        );
    }
    Ok(())
}
