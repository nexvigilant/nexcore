//! nexcore-vad — Voice Activity Detection CLI
//!
//! Reads raw F32 mono 16kHz audio from stdin, runs the VoiceDetector,
//! and outputs JSON lines with VAD state for each frame.
//!
//! ## Usage
//!
//! ```bash
//! # Pipe raw audio from PipeWire/PulseAudio:
//! parec --format=float32le --rate=16000 --channels=1 | nexcore-vad
//!
//! # From a WAV file (strip header first):
//! ffmpeg -i input.wav -f f32le -ar 16000 -ac 1 pipe:1 | nexcore-vad
//!
//! # With custom config:
//! nexcore-vad --frame-ms 20 --onset 2 --offset 10 --zcr-ceiling 0.35
//! ```
//!
//! ## Output (JSON lines)
//!
//! ```json
//! {"speech":false,"energy":0.003,"zcr":0.12,"threshold":0.015,"state":"Silence"}
//! {"speech":true,"energy":0.18,"zcr":0.09,"threshold":0.015,"state":"Speech"}
//! ```
//!
//! ## Integration with vigil-listen
//!
//! vigil-listen pipes its sounddevice capture through this binary.
//! The Python side reads JSON lines and uses `speech` to gate transcription.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::{self, Read, Write};

use nexcore_audio::vad::{VadConfig, VoiceDetector};

const SAMPLE_RATE: usize = 16000;
const DEFAULT_FRAME_MS: usize = 30;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = parse_config(&args);
    let frame_samples = SAMPLE_RATE * config.frame_ms / 1000;
    let frame_bytes = frame_samples * 4; // f32 = 4 bytes

    let mut detector = VoiceDetector::new(config.vad);
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut buf = vec![0u8; frame_bytes];

    // Optional: emit config on stderr for debugging
    if config.verbose {
        eprintln!(
            "nexcore-vad: frame={}ms ({}samples), rate={}Hz",
            config.frame_ms, frame_samples, SAMPLE_RATE
        );
    }

    loop {
        match stdin.read_exact(&mut buf) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("nexcore-vad: read error: {e}");
                break;
            }
        }

        // Convert bytes to f32 samples (little-endian)
        let frame: Vec<f32> = buf
            .chunks_exact(4)
            .map(|chunk| {
                let bytes: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
                f32::from_le_bytes(bytes)
            })
            .collect();

        let result = detector.process(&frame);

        // Output compact JSON line
        let line = format!(
            "{{\"speech\":{},\"energy\":{:.4},\"zcr\":{:.3},\"threshold\":{:.4},\"state\":\"{}\"}}",
            result.is_speech,
            result.energy,
            result.zcr,
            result.threshold,
            match result.state {
                nexcore_audio::vad::VadState::Silence => "Silence",
                nexcore_audio::vad::VadState::Onset => "Onset",
                nexcore_audio::vad::VadState::Speech => "Speech",
                nexcore_audio::vad::VadState::Offset => "Offset",
            }
        );

        if let Err(e) = writeln!(stdout, "{line}") {
            if e.kind() == io::ErrorKind::BrokenPipe {
                break;
            }
            eprintln!("nexcore-vad: write error: {e}");
            break;
        }
    }

    if config.verbose {
        eprintln!(
            "nexcore-vad: processed {} frames, final state: {:?}",
            detector.frame_count(),
            detector.state()
        );
    }
}

struct CliConfig {
    vad: VadConfig,
    frame_ms: usize,
    verbose: bool,
}

fn parse_config(args: &[String]) -> CliConfig {
    let mut config = CliConfig {
        vad: VadConfig::default(),
        frame_ms: DEFAULT_FRAME_MS,
        verbose: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--frame-ms" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|s| s.parse().ok()) {
                    config.frame_ms = val;
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
            "--energy" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|s| s.parse::<f32>().ok()) {
                    config.vad.energy_threshold = val;
                    // Also set min_energy so adaptive floor never drops below calibrated value
                    config.vad.min_energy = val * 0.5;
                }
            }
            "--zcr-ceiling" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|s| s.parse().ok()) {
                    config.vad.zcr_ceiling = val;
                }
            }
            "--adapt-rate" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|s| s.parse().ok()) {
                    config.vad.adaptation_rate = val;
                }
            }
            "-v" | "--verbose" => {
                config.verbose = true;
            }
            "--help" | "-h" => {
                eprintln!("nexcore-vad — Voice Activity Detection (Rust)");
                eprintln!();
                eprintln!("Reads raw F32LE mono 16kHz audio from stdin.");
                eprintln!("Outputs JSON lines with VAD state per frame.");
                eprintln!();
                eprintln!("Options:");
                eprintln!("  --frame-ms N      Frame size in ms (default: 30)");
                eprintln!("  --onset N         Frames to confirm speech onset (default: 3)");
                eprintln!("  --offset N        Frames to confirm speech offset (default: 15)");
                eprintln!("  --energy F        Initial energy threshold (default: 0.02)");
                eprintln!("  --zcr-ceiling F   Max zero-crossing rate for speech (default: 0.4)");
                eprintln!("  --adapt-rate F    Noise floor adaptation rate (default: 0.995)");
                eprintln!("  -v, --verbose     Print debug info to stderr");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    config
}
