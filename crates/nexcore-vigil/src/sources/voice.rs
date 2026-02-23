use crate::events::EventBus;
use crate::models::{Event, Urgency};
use crate::sources::Source;
use async_trait::async_trait;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub struct VoiceSource {
    bus: EventBus,
    cancel_token: CancellationToken,
}

impl VoiceSource {
    pub fn new(bus: EventBus) -> Self {
        Self {
            bus,
            cancel_token: CancellationToken::new(),
        }
    }

    pub fn shutdown(&self) {
        self.cancel_token.cancel();
    }
}

#[async_trait]
impl Source for VoiceSource {
    fn name(&self) -> &'static str {
        "voice"
    }

    async fn run(&self) -> nexcore_error::Result<()> {
        info!("voice_source_starting");
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| nexcore_error::nexerror!("no_input_device"))?;
        let config: cpal::StreamConfig = device.default_input_config()?.into();
        let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let buffer_clone = audio_buffer.clone();
        let cancel = self.cancel_token.clone();

        std::thread::spawn(move || {
            let stream = match device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    if let Ok(mut b) = buffer_clone.try_lock() {
                        b.extend_from_slice(data);
                    }
                },
                |err| {
                    error!(error = %err, "audio_stream_error");
                },
                None,
            ) {
                Ok(stream) => stream,
                Err(err) => {
                    error!(error = %err, "stream_build_fail");
                    return;
                }
            };
            if let Err(err) = stream.play() {
                error!(error = %err, "stream_play_fail");
                return;
            }
            while !cancel.is_cancelled() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            info!("audio_stream_stopped");
        });

        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => break,
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                    let mut buf = audio_buffer.lock().await;
                    if buf.len() > 16000 * 2 {
                        self.bus.emit(Event {
                            source: "voice".to_string(),
                            event_type: "user_spoke".to_string(),
                            payload: serde_json::json!({ "text": "Simulated voice" }),
                            priority: Urgency::High,
                            ..Event::default()
                        }).await;
                        buf.clear();
                    }
                }
            }
        }
        Ok(())
    }
}
