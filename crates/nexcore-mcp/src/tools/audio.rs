//! Audio MCP tools — sample conversion, spec computation, codec catalog.
//!
//! Pure-function wrappers for nexcore-audio: sample format conversion,
//! audio spec properties, resampling, codec info, pan law, stream states.

use nexcore_audio::codec::{CodecId, ResampleQuality};
use nexcore_audio::sample::{AudioSpec, ChannelLayout, SampleFormat, SampleRate};
use nexcore_audio::stream::StreamState;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::audio::{
    AudioCodecCatalogParams, AudioConvertSampleParams, AudioDeviceCapabilitiesParams,
    AudioFormatInfoParams, AudioMixerPanParams, AudioRateInfoParams, AudioResampleParams,
    AudioSpecComputeParams, AudioSpecPresetsParams, AudioStreamTransitionsParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn parse_format(s: &str) -> Option<SampleFormat> {
    match s.to_lowercase().trim() {
        "s16" | "i16" | "int16" => Some(SampleFormat::S16),
        "s24" | "i24" | "int24" => Some(SampleFormat::S24),
        "s32" | "i32" | "int32" => Some(SampleFormat::S32),
        "f32" | "float32" | "float" => Some(SampleFormat::F32),
        "u8" | "uint8" => Some(SampleFormat::U8),
        _ => None,
    }
}

fn parse_layout(s: &str) -> Option<ChannelLayout> {
    match s.to_lowercase().trim() {
        "mono" | "1" => Some(ChannelLayout::Mono),
        "stereo" | "2" => Some(ChannelLayout::Stereo),
        "surround_21" | "2.1" | "3" => Some(ChannelLayout::Surround21),
        "surround_51" | "5.1" | "6" => Some(ChannelLayout::Surround51),
        "surround_71" | "7.1" | "8" => Some(ChannelLayout::Surround71),
        other => other.parse::<u16>().ok().map(ChannelLayout::Custom),
    }
}

fn parse_stream_state(s: &str) -> Option<StreamState> {
    match s.to_lowercase().trim() {
        "created" => Some(StreamState::Created),
        "running" => Some(StreamState::Running),
        "paused" => Some(StreamState::Paused),
        "stopped" => Some(StreamState::Stopped),
        "error" => Some(StreamState::Error),
        _ => None,
    }
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Compute audio spec properties (bytes/frame, bytes/sec, duration).
pub fn audio_spec_compute(p: AudioSpecComputeParams) -> Result<CallToolResult, McpError> {
    let format = match parse_format(&p.format) {
        Some(f) => f,
        None => return err_result("format must be s16, s24, s32, f32, or u8"),
    };
    let rate = SampleRate::from_hz(p.rate);
    let layout = match parse_layout(&p.layout) {
        Some(l) => l,
        None => {
            return err_result(
                "layout must be mono, stereo, surround_51, surround_71, or a channel count",
            );
        }
    };

    let spec = AudioSpec::new(format, rate, layout);

    let mut result = json!({
        "format": format.name(),
        "rate_hz": rate.hz(),
        "channels": layout.channels(),
        "bytes_per_frame": spec.bytes_per_frame(),
        "bytes_per_second": spec.bytes_per_second(),
    });

    if let Some(bytes) = p.bytes {
        result["duration_secs"] = json!(spec.duration_secs(bytes));
    }
    if let Some(secs) = p.duration_secs {
        result["bytes_for_duration"] = json!(spec.bytes_for_duration(secs));
    }

    ok_json(result)
}

/// List standard audio spec presets.
pub fn audio_spec_presets(_p: AudioSpecPresetsParams) -> Result<CallToolResult, McpError> {
    let presets = [
        ("cd_quality", AudioSpec::cd_quality()),
        ("dvd_quality", AudioSpec::dvd_quality()),
        ("voice_quality", AudioSpec::voice_quality()),
        ("float_stereo", AudioSpec::float_stereo()),
    ];

    let items: Vec<serde_json::Value> = presets
        .iter()
        .map(|(name, spec)| {
            json!({
                "name": name,
                "format": spec.format.name(),
                "rate_hz": spec.rate.hz(),
                "channels": spec.layout.channels(),
                "bytes_per_frame": spec.bytes_per_frame(),
                "bytes_per_second": spec.bytes_per_second(),
            })
        })
        .collect();

    ok_json(json!({ "presets": items }))
}

/// Get properties of a sample format.
pub fn audio_format_info(p: AudioFormatInfoParams) -> Result<CallToolResult, McpError> {
    let format = match parse_format(&p.format) {
        Some(f) => f,
        None => return err_result("format must be s16, s24, s32, f32, or u8"),
    };

    ok_json(json!({
        "name": format.name(),
        "bytes_per_sample": format.bytes_per_sample(),
        "bits_per_sample": format.bits_per_sample(),
        "is_float": format.is_float(),
        "is_integer": format.is_integer(),
    }))
}

/// Get properties of a sample rate.
pub fn audio_rate_info(p: AudioRateInfoParams) -> Result<CallToolResult, McpError> {
    let rate = SampleRate::from_hz(p.rate);
    ok_json(json!({
        "hz": rate.hz(),
        "period_us": rate.period_us(),
        "is_standard": rate.is_standard(),
    }))
}

/// Convert a single audio sample between formats.
pub fn audio_convert_sample(p: AudioConvertSampleParams) -> Result<CallToolResult, McpError> {
    use nexcore_audio::codec;

    let result = match (p.from.to_lowercase().as_str(), p.to.to_lowercase().as_str()) {
        ("s16", "f32") => {
            let v = codec::s16_to_f32(p.value as i16);
            json!({"from": "s16", "to": "f32", "input": p.value as i16, "output": v})
        }
        ("f32", "s16") => {
            let v = codec::f32_to_s16(p.value as f32);
            json!({"from": "f32", "to": "s16", "input": p.value as f32, "output": v})
        }
        ("u8", "f32") => {
            let v = codec::u8_to_f32(p.value as u8);
            json!({"from": "u8", "to": "f32", "input": p.value as u8, "output": v})
        }
        ("f32", "u8") => {
            let v = codec::f32_to_u8(p.value as f32);
            json!({"from": "f32", "to": "u8", "input": p.value as f32, "output": v})
        }
        _ => return err_result("conversion must be s16→f32, f32→s16, u8→f32, or f32→u8"),
    };

    ok_json(result)
}

/// Resample an F32 audio buffer between sample rates.
pub fn audio_resample(p: AudioResampleParams) -> Result<CallToolResult, McpError> {
    let from_rate = SampleRate::from_hz(p.from_rate);
    let to_rate = SampleRate::from_hz(p.to_rate);
    let quality = match p.quality.as_deref().unwrap_or("linear") {
        "nearest" => ResampleQuality::Nearest,
        "linear" => ResampleQuality::Linear,
        _ => return err_result("quality must be 'nearest' or 'linear'"),
    };

    let result = nexcore_audio::codec::resample_f32(&p.samples, from_rate, to_rate, quality);

    ok_json(json!({
        "from_rate": from_rate.hz(),
        "to_rate": to_rate.hz(),
        "quality": format!("{quality:?}"),
        "input_samples": p.samples.len(),
        "output_samples": result.len(),
        "samples": result,
    }))
}

/// List all audio codec types.
pub fn audio_codec_catalog(_p: AudioCodecCatalogParams) -> Result<CallToolResult, McpError> {
    let codecs = [CodecId::Pcm, CodecId::MuLaw, CodecId::ALaw];
    let items: Vec<serde_json::Value> = codecs
        .iter()
        .map(|c| {
            json!({
                "id": format!("{c:?}"),
                "name": c.name(),
            })
        })
        .collect();
    ok_json(json!({ "codecs": items }))
}

/// Check device capabilities — preferred spec and format support.
pub fn audio_device_capabilities(
    p: AudioDeviceCapabilitiesParams,
) -> Result<CallToolResult, McpError> {
    let formats: Vec<SampleFormat> = p.formats.iter().filter_map(|s| parse_format(s)).collect();
    let rates: Vec<SampleRate> = p.rates.iter().map(|&hz| SampleRate::from_hz(hz)).collect();
    let layouts: Vec<ChannelLayout> = p.layouts.iter().filter_map(|s| parse_layout(s)).collect();

    let caps = nexcore_audio::device::DeviceCapabilities {
        formats: formats.clone(),
        rates: rates.clone(),
        layouts: layouts.clone(),
        min_buffer_frames: p.min_buffer_frames.unwrap_or(256),
        max_buffer_frames: p.max_buffer_frames.unwrap_or(8192),
    };

    let preferred = caps.preferred_spec();

    ok_json(json!({
        "formats": formats.iter().map(|f| f.name()).collect::<Vec<_>>(),
        "rates_hz": rates.iter().map(|r| r.hz()).collect::<Vec<_>>(),
        "channels": layouts.iter().map(|l| l.channels()).collect::<Vec<_>>(),
        "min_buffer_frames": caps.min_buffer_frames,
        "max_buffer_frames": caps.max_buffer_frames,
        "preferred_spec": preferred.map(|spec| json!({
            "format": spec.format.name(),
            "rate_hz": spec.rate.hz(),
            "channels": spec.layout.channels(),
        })),
    }))
}

/// Compute stereo pan gains using constant-power pan law.
pub fn audio_mixer_pan(p: AudioMixerPanParams) -> Result<CallToolResult, McpError> {
    let mut source = nexcore_audio::mixer::MixerSource::new("pan_calc");
    source.set_pan(p.pan);
    if let Some(vol) = p.volume {
        source.set_volume(vol);
    }
    if let Some(muted) = p.muted {
        source.set_muted(muted);
    }

    ok_json(json!({
        "pan": source.pan(),
        "volume": source.volume(),
        "muted": source.is_muted(),
        "effective_volume": source.effective_volume(),
        "left_gain": source.left_gain(),
        "right_gain": source.right_gain(),
    }))
}

/// Get available state transitions for an audio stream state.
pub fn audio_stream_transitions(
    p: AudioStreamTransitionsParams,
) -> Result<CallToolResult, McpError> {
    let state = match parse_stream_state(&p.state) {
        Some(s) => s,
        None => return err_result("state must be created, running, paused, stopped, or error"),
    };

    ok_json(json!({
        "state": format!("{state:?}"),
        "is_active": state.is_active(),
        "can_start": state.can_start(),
        "can_pause": state.can_pause(),
        "can_stop": state.can_stop(),
    }))
}
