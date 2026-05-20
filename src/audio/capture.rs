//! Loopback / microphone capture via `cpal` (Phase 1 scaffold).
//!
//! Full speech-to-text pipeline is deferred; this module establishes the async listener
//! and logs buffer statistics for future transcript integration.

use crate::config::AudioConfig;
use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn};

static AUDIO_RUNNING: AtomicBool = AtomicBool::new(false);

/// Start a non-blocking audio capture loop on a background thread.
pub fn spawn_audio_listener(config: AudioConfig) {
    if !config.enabled {
        info!("audio capture disabled in config");
        return;
    }
    if AUDIO_RUNNING.swap(true, Ordering::SeqCst) {
        warn!("audio listener already running");
        return;
    }
    std::thread::spawn(move || {
        if let Err(e) = run_capture_loop(&config) {
            tracing::error!("audio capture ended: {e:#}");
        }
        AUDIO_RUNNING.store(false, Ordering::SeqCst);
    });
}

fn run_capture_loop(config: &AudioConfig) -> Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("no default input audio device"))?;
    info!("audio device: {}", device.name()?);

    let supported = device
        .default_input_config()
        .map_err(|e| anyhow::anyhow!("input config: {e}"))?;
    let sample_rate = supported.sample_rate().0;
    info!(
        "audio stream: {:?} @ {} Hz (hint {} Hz)",
        supported.sample_format(),
        sample_rate,
        config.sample_rate_hint
    );

    let err_fn = |err| tracing::error!("audio stream error: {err}");
    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &supported.into(), err_fn)?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &supported.into(), err_fn)?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &supported.into(), err_fn)?,
        other => anyhow::bail!("unsupported sample format: {other:?}"),
    };

    stream.play()?;
    info!("audio capture active — transcript pipeline TBD");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream>
where
    T: cpal::SizedSample + cpal::Sample + cpal::FromSample<f32>,
    f32: cpal::FromSample<T>,
{
    let peak = std::sync::Arc::new(std::sync::Mutex::new(0.0f32));
    let peak_clone = peak.clone();
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _| {
            let mut max = 0.0f32;
            for s in data {
                let v = cpal::Sample::to_sample::<f32>(*s).abs();
                if v > max {
                    max = v;
                }
            }
            let mut guard = peak_clone.lock().unwrap();
            if max > *guard {
                *guard = max;
            }
        },
        err_fn,
        None,
    )?;
    Ok(stream)
}
