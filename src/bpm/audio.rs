use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;

const FRAME_SIZE: usize = 1024;
const MAX_ONSETS: usize = 16;
const MIN_ONSET_GAP_SECS: f64 = 0.2;
const ENERGY_HISTORY_LEN: usize = 43;
const MIN_BPM: f64 = 40.0;
const MAX_BPM: f64 = 220.0;
const DEFAULT_THRESHOLD: f32 = 1.5;

struct AudioState {
    current_bpm: Option<f64>,
    is_listening: bool,
    onset_timestamps: VecDeque<Instant>,
    energy_history: VecDeque<f32>,
    last_onset: Option<Instant>,
    error: Option<String>,
    sample_buffer: Vec<f32>,
    threshold_multiplier: f32,
}

impl AudioState {
    fn new() -> Self {
        Self {
            current_bpm: None,
            is_listening: false,
            onset_timestamps: VecDeque::with_capacity(MAX_ONSETS + 1),
            energy_history: VecDeque::with_capacity(ENERGY_HISTORY_LEN + 1),
            last_onset: None,
            error: None,
            sample_buffer: Vec::with_capacity(FRAME_SIZE),
            threshold_multiplier: DEFAULT_THRESHOLD,
        }
    }

    fn reset(&mut self) {
        self.current_bpm = None;
        self.onset_timestamps.clear();
        self.energy_history.clear();
        self.last_onset = None;
        self.sample_buffer.clear();
        self.error = None;
    }

    fn process_samples(&mut self, samples: &[f32]) {
        self.sample_buffer.extend_from_slice(samples);

        while self.sample_buffer.len() >= FRAME_SIZE {
            let frame: Vec<f32> = self.sample_buffer.drain(..FRAME_SIZE).collect();
            self.process_frame(&frame);
        }
    }

    fn process_frame(&mut self, frame: &[f32]) {
        let rms = (frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32).sqrt();

        self.energy_history.push_back(rms);
        if self.energy_history.len() > ENERGY_HISTORY_LEN {
            self.energy_history.pop_front();
        }

        if self.energy_history.len() < 4 {
            return;
        }

        let avg_energy =
            self.energy_history.iter().sum::<f32>() / self.energy_history.len() as f32;

        if rms <= avg_energy * self.threshold_multiplier || avg_energy <= 0.001 {
            return;
        }

        let now = Instant::now();
        if let Some(last) = self.last_onset {
            if now.duration_since(last).as_secs_f64() < MIN_ONSET_GAP_SECS {
                return;
            }
        }

        self.last_onset = Some(now);
        self.onset_timestamps.push_back(now);
        if self.onset_timestamps.len() > MAX_ONSETS {
            self.onset_timestamps.pop_front();
        }

        self.update_bpm();
    }

    fn update_bpm(&mut self) {
        if self.onset_timestamps.len() < 3 {
            return;
        }

        let intervals: Vec<f64> = self
            .onset_timestamps
            .iter()
            .zip(self.onset_timestamps.iter().skip(1))
            .map(|(a, b)| b.duration_since(*a).as_secs_f64())
            .collect();

        let avg_interval = intervals.iter().sum::<f64>() / intervals.len() as f64;

        if avg_interval > 0.0 {
            let bpm = 60.0 / avg_interval;
            if bpm >= MIN_BPM && bpm <= MAX_BPM {
                self.current_bpm = Some(bpm);
            }
        }
    }
}

static SHARED_STATE: LazyLock<Arc<Mutex<AudioState>>> =
    LazyLock::new(|| Arc::new(Mutex::new(AudioState::new())));

static STREAM: Mutex<Option<cpal::Stream>> = Mutex::new(None);

pub fn start_listening() -> Result<(), String> {
    let mut stream_guard = STREAM.lock().map_err(|e| e.to_string())?;

    if stream_guard.is_some() {
        let still_listening = SHARED_STATE
            .lock()
            .map(|s| s.is_listening)
            .unwrap_or(false);
        if still_listening {
            return Ok(());
        }
        *stream_guard = None;
    }

    {
        let mut state = SHARED_STATE.lock().map_err(|e| e.to_string())?;
        state.reset();
        state.is_listening = true;
    }

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available. Check microphone permissions.")?;

    let config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get input config: {e}"))?;

    let channels = config.channels() as usize;
    let shared = Arc::clone(&SHARED_STATE);
    let err_shared = Arc::clone(&SHARED_STATE);

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mono: Vec<f32> = if channels == 1 {
                    data.to_vec()
                } else {
                    data.chunks(channels).map(|c| c[0]).collect()
                };
                if let Ok(mut s) = shared.try_lock() {
                    s.process_samples(&mono);
                }
            },
            move |err| {
                if let Ok(mut s) = err_shared.try_lock() {
                    s.error = Some(format!("Audio error: {err}"));
                    s.is_listening = false;
                }
            },
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                let mono: Vec<f32> = if channels == 1 {
                    data.iter().map(|&s| s as f32 / i16::MAX as f32).collect()
                } else {
                    data.chunks(channels)
                        .map(|c| c[0] as f32 / i16::MAX as f32)
                        .collect()
                };
                if let Ok(mut s) = shared.try_lock() {
                    s.process_samples(&mono);
                }
            },
            move |err| {
                if let Ok(mut s) = err_shared.try_lock() {
                    s.error = Some(format!("Audio error: {err}"));
                    s.is_listening = false;
                }
            },
            None,
        ),
        format => return Err(format!("Unsupported sample format: {format:?}")),
    }
    .map_err(|e| format!("Failed to build input stream: {e}"))?;

    stream
        .play()
        .map_err(|e| format!("Failed to start stream: {e}"))?;

    *stream_guard = Some(stream);
    Ok(())
}

pub fn stop_listening() {
    if let Ok(mut stream_guard) = STREAM.lock() {
        *stream_guard = None;
    }
    if let Ok(mut state) = SHARED_STATE.lock() {
        state.is_listening = false;
    }
}

pub fn current_mic_bpm() -> Option<f64> {
    SHARED_STATE.lock().ok()?.current_bpm
}

pub fn is_mic_listening() -> bool {
    SHARED_STATE
        .lock()
        .map(|s| s.is_listening)
        .unwrap_or(false)
}
