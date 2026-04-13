//! nexcore-transcribe — Transcribe a WAV file via faster-whisper
//!
//! Single-shot file transcription. Takes a WAV path, outputs JSON.
//!
//! ## Usage
//!
//! ```bash
//! nexcore-transcribe /tmp/audio.wav
//! nexcore-transcribe --model small.en --prompt "NexVigilant" /tmp/audio.wav
//! ```
//!
//! ## Output
//!
//! ```json
//! {"text":"hello world","confidence":-0.21,"duration_secs":1.5,"accepted":true,"language":"en"}
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use nexcore_audio::stt::{SttConfig, transcribe_file};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (config, path) = parse_args(&args);

    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("Usage: nexcore-transcribe [options] <file.wav>");
            std::process::exit(1);
        }
    };

    match transcribe_file(Path::new(&path), &config) {
        Ok(transcript) => match serde_json::to_string(&transcript) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("serialization error: {e}");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn parse_args(args: &[String]) -> (SttConfig, Option<String>) {
    let mut config = SttConfig::default();
    let mut path = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--model" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    config.model = v.clone();
                }
            }
            "--lang" | "--language" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    config.language = v.clone();
                }
            }
            "--prompt" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    config.initial_prompt = v.clone();
                }
            }
            "--beam" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse().ok()) {
                    config.beam_size = v;
                }
            }
            "-h" | "--help" => {
                eprintln!("nexcore-transcribe — WAV file transcription via faster-whisper");
                eprintln!("Usage: nexcore-transcribe [options] <file.wav>");
                eprintln!("  --model NAME    Whisper model (default: medium.en)");
                eprintln!("  --lang CODE     Language (default: en)");
                eprintln!("  --prompt TEXT    Vocabulary priming");
                eprintln!("  --beam N        Beam size (default: 3)");
                std::process::exit(0);
            }
            other => {
                if !other.starts_with("--") {
                    path = Some(other.to_string());
                }
            }
        }
        i += 1;
    }

    (config, path)
}
