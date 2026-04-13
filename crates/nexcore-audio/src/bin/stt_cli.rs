//! nexcore-stt — Mic-to-text pipeline: NoiseGate → VAD → STT
//!
//! Reads raw F32 mono 16kHz audio from stdin, chains through the noise gate
//! and voice activity detector, accumulates speech segments, transcribes
//! via faster-whisper, and outputs text to stdout.
//!
//! ## Usage
//!
//! ```bash
//! # From PipeWire mic:
//! parec --format=float32le --rate=16000 --channels=1 | nexcore-stt
//!
//! # From file:
//! ffmpeg -i input.wav -f f32le -ar 16000 -ac 1 pipe:1 | nexcore-stt
//!
//! # With options:
//! nexcore-stt --model small.en --energy 0.10 --prompt "NexVigilant pharmacovigilance"
//! ```
//!
//! ## Output
//!
//! Each transcribed utterance is printed as a JSON line:
//! ```json
//! {"text":"hello vigil","confidence":-0.21,"duration":1.8,"accepted":true}
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::{self, Read, Write};
use std::path::PathBuf;

use nexcore_audio::noise::{NoiseGate, NoiseGateConfig};
use nexcore_audio::stt::{SttConfig, transcribe_file};
use nexcore_audio::vad::{VadConfig, VadState, VoiceDetector};

const SAMPLE_RATE: u32 = 16000;
const FRAME_MS: usize = 30;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = parse_config(&args);

    let frame_samples = (SAMPLE_RATE as usize) * FRAME_MS / 1000;
    let frame_bytes = frame_samples * 4; // f32 = 4 bytes

    let mut vad = VoiceDetector::new(config.vad.clone());
    let mut gate = NoiseGate::new(config.gate.clone());
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut buf = vec![0u8; frame_bytes];

    // Speech accumulator
    let mut speech_buffer: Vec<f32> = Vec::new();
    let mut utterance_count: u64 = 0;

    if config.verbose {
        eprintln!(
            "nexcore-stt: pipeline active (frame={}ms, model={}, energy={})",
            FRAME_MS, config.stt.model, config.vad.energy_threshold
        );
    }

    loop {
        match stdin.read_exact(&mut buf) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // Flush remaining speech
                if !speech_buffer.is_empty() {
                    transcribe_and_emit(
                        &speech_buffer,
                        &config.stt,
                        &mut stdout,
                        &mut utterance_count,
                        config.verbose,
                    );
                }
                break;
            }
            Err(e) => {
                eprintln!("nexcore-stt: read error: {e}");
                break;
            }
        }

        // Decode F32LE
        let frame: Vec<f32> = buf
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        // Stage 1: Noise Gate
        let (cleaned, _gate_result) = gate.process(&frame);

        // Stage 2: VAD
        let vad_result = vad.process(&cleaned);

        match vad_result.state {
            VadState::Onset | VadState::Speech | VadState::Offset => {
                // Accumulate speech frames (use cleaned audio)
                speech_buffer.extend_from_slice(&cleaned);

                // Hard cap: 30 seconds max utterance
                let max_samples = SAMPLE_RATE as usize * 30;
                if speech_buffer.len() >= max_samples {
                    transcribe_and_emit(
                        &speech_buffer,
                        &config.stt,
                        &mut stdout,
                        &mut utterance_count,
                        config.verbose,
                    );
                    speech_buffer.clear();
                }
            }
            VadState::Silence => {
                // VAD confirmed silence after offset period — transcribe accumulated speech
                if !speech_buffer.is_empty() {
                    // Minimum duration: 0.3s (avoid transcribing clicks)
                    let min_samples = (SAMPLE_RATE as f32 * 0.3) as usize;
                    if speech_buffer.len() >= min_samples {
                        transcribe_and_emit(
                            &speech_buffer,
                            &config.stt,
                            &mut stdout,
                            &mut utterance_count,
                            config.verbose,
                        );
                    } else if config.verbose {
                        eprintln!(
                            "nexcore-stt: dropped short segment ({:.1}ms)",
                            speech_buffer.len() as f64 / SAMPLE_RATE as f64 * 1000.0
                        );
                    }
                    speech_buffer.clear();
                }
            }
        }
    }

    if config.verbose {
        eprintln!("nexcore-stt: {utterance_count} utterances transcribed");
    }
}

/// Write speech buffer to temp WAV, transcribe, emit JSON line.
fn transcribe_and_emit(
    samples: &[f32],
    stt_config: &SttConfig,
    stdout: &mut impl Write,
    count: &mut u64,
    verbose: bool,
) {
    let duration_secs = samples.len() as f64 / SAMPLE_RATE as f64;
    if verbose {
        eprintln!("nexcore-stt: transcribing {duration_secs:.1}s utterance...");
    }

    // Write temp WAV
    let tmp_path = PathBuf::from(format!("/tmp/nexcore-stt-{}.wav", std::process::id()));
    if write_wav(&tmp_path, samples).is_err() {
        eprintln!("nexcore-stt: failed to write temp WAV");
        return;
    }

    match transcribe_file(&tmp_path, stt_config) {
        Ok(transcript) => {
            if transcript.accepted && !transcript.text.is_empty() {
                // Emit compact JSON line
                let line = format!(
                    "{{\"text\":{},\"confidence\":{:.3},\"duration\":{:.1},\"accepted\":true}}",
                    serde_json::to_string(&transcript.text).unwrap_or_else(|_| "\"\"".to_string()),
                    transcript.confidence,
                    transcript.duration_secs,
                );
                if let Err(e) = writeln!(stdout, "{line}") {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        std::process::exit(0);
                    }
                }
                *count = count.saturating_add(1);
            } else if verbose {
                eprintln!(
                    "nexcore-stt: rejected (conf={:.2}, text='{}')",
                    transcript.confidence,
                    &transcript.text[..transcript.text.len().min(50)]
                );
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("nexcore-stt: STT error: {e}");
            }
        }
    }

    // Cleanup
    std::fs::remove_file(&tmp_path).ok();
}

/// Write F32 samples to a 16kHz mono S16 WAV file.
fn write_wav(path: &PathBuf, samples: &[f32]) -> Result<(), io::Error> {
    use std::io::Cursor;

    let num_samples = samples.len();
    let data_size = (num_samples * 2) as u32; // S16 = 2 bytes/sample
    let file_size = 36 + data_size;

    let mut buf = Vec::with_capacity(file_size as usize + 8);
    let mut cursor = Cursor::new(&mut buf);

    // RIFF header
    cursor.write_all(b"RIFF")?;
    cursor.write_all(&file_size.to_le_bytes())?;
    cursor.write_all(b"WAVE")?;

    // fmt chunk
    cursor.write_all(b"fmt ")?;
    cursor.write_all(&16u32.to_le_bytes())?; // chunk size
    cursor.write_all(&1u16.to_le_bytes())?; // PCM
    cursor.write_all(&1u16.to_le_bytes())?; // mono
    cursor.write_all(&SAMPLE_RATE.to_le_bytes())?; // sample rate
    cursor.write_all(&(SAMPLE_RATE * 2).to_le_bytes())?; // byte rate
    cursor.write_all(&2u16.to_le_bytes())?; // block align
    cursor.write_all(&16u16.to_le_bytes())?; // bits per sample

    // data chunk
    cursor.write_all(b"data")?;
    cursor.write_all(&data_size.to_le_bytes())?;

    // Convert F32 to S16
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let i16_val = (clamped * 32767.0) as i16;
        cursor.write_all(&i16_val.to_le_bytes())?;
    }

    std::fs::write(path, buf)?;
    Ok(())
}

struct PipelineConfig {
    vad: VadConfig,
    gate: NoiseGateConfig,
    stt: SttConfig,
    verbose: bool,
}

fn parse_config(args: &[String]) -> PipelineConfig {
    let mut config = PipelineConfig {
        vad: VadConfig::default(),
        gate: NoiseGateConfig::default(),
        stt: SttConfig::default(),
        verbose: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--energy" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|s| s.parse::<f32>().ok()) {
                    config.vad.energy_threshold = val;
                    config.vad.min_energy = val * 0.5;
                }
            }
            "--zcr-ceiling" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|s| s.parse().ok()) {
                    config.vad.zcr_ceiling = val;
                }
            }
            "--onset" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|s| s.parse().ok()) {
                    config.vad.onset_frames = val;
                }
            }
            "--offset" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|s| s.parse().ok()) {
                    config.vad.offset_frames = val;
                }
            }
            "--model" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    config.stt.model = val.clone();
                }
            }
            "--language" | "--lang" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    config.stt.language = val.clone();
                }
            }
            "--prompt" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    config.stt.initial_prompt = val.clone();
                }
            }
            "--gate-threshold" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|s| s.parse().ok()) {
                    config.gate.threshold_multiplier = val;
                }
            }
            "-v" | "--verbose" => {
                config.verbose = true;
            }
            "--help" | "-h" => {
                eprintln!("nexcore-stt — Mic-to-text pipeline (NoiseGate → VAD → STT)");
                eprintln!();
                eprintln!("Reads raw F32LE mono 16kHz from stdin, outputs JSON transcript lines.");
                eprintln!();
                eprintln!("Options:");
                eprintln!("  --energy F          VAD energy threshold (default: 0.02)");
                eprintln!("  --zcr-ceiling F     Max ZCR for speech (default: 0.4)");
                eprintln!("  --onset N           Frames to confirm speech (default: 3)");
                eprintln!("  --offset N          Frames to confirm silence (default: 15)");
                eprintln!("  --model NAME        Whisper model (default: medium.en)");
                eprintln!("  --lang CODE         Language hint (default: en)");
                eprintln!("  --prompt TEXT        Vocabulary priming prompt");
                eprintln!("  --gate-threshold F  Noise gate multiplier (default: 2.0)");
                eprintln!("  -v, --verbose       Debug output to stderr");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    config
}
