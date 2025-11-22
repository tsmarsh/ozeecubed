use std::collections::VecDeque;
use std::time::Instant;
use winit::keyboard::KeyCode;

use ozeecubed_core::audio::AudioCapture;
use ozeecubed_core::oscilloscope::{TriggerSettings, WaveformData};

const PERSISTENCE_FRAMES: usize = 10;

pub struct AppState {
    pub waveform: WaveformData,
    pub trigger_settings: TriggerSettings,
    pub waveform_history: VecDeque<Vec<(f32, f32)>>,
    audio_capture: Option<AudioCapture>,
    audio_buffer: Vec<f32>,
    last_update: Instant,
    frame_count: usize,
}

impl AppState {
    pub fn new() -> Self {
        let waveform = WaveformData::new(48000);
        let trigger_settings = TriggerSettings::default();

        // Try to initialize audio capture
        let audio_capture = match AudioCapture::new() {
            Ok(capture) => {
                println!("Audio capture initialized");
                Some(capture)
            }
            Err(e) => {
                eprintln!("Failed to initialize audio capture: {e}");
                eprintln!("Using test signal");
                None
            }
        };

        Self {
            waveform,
            trigger_settings,
            waveform_history: VecDeque::new(),
            audio_capture,
            audio_buffer: Vec::new(),
            last_update: Instant::now(),
            frame_count: 0,
        }
    }

    pub fn update(&mut self) {
        // Update at ~60 FPS
        let now = Instant::now();
        if now.duration_since(self.last_update).as_millis() < 16 {
            return;
        }
        self.last_update = now;

        // Read audio samples
        if let Some(ref audio_capture) = self.audio_capture {
            let new_samples = audio_capture.read_samples(usize::MAX);
            if !new_samples.is_empty() {
                self.audio_buffer.extend_from_slice(&new_samples);

                let samples_needed = self.waveform.calculate_samples_per_screen();
                let max_buffer_size = samples_needed + 200;

                if self.audio_buffer.len() > max_buffer_size {
                    let to_remove = self.audio_buffer.len() - max_buffer_size;
                    self.audio_buffer.drain(0..to_remove);
                }

                self.waveform.update_samples(self.audio_buffer.clone());
            }
        } else {
            // Generate test signal
            self.generate_test_signal();
        }

        // Update persistence history every frame
        self.frame_count += 1;
        let points = self.waveform.get_display_samples(&self.trigger_settings);
        self.add_to_history(points);
    }

    fn add_to_history(&mut self, points: Vec<(f32, f32)>) {
        if !points.is_empty() {
            self.waveform_history.push_back(points);
            if self.waveform_history.len() > PERSISTENCE_FRAMES {
                self.waveform_history.pop_front();
            }
        }
    }

    fn generate_test_signal(&mut self) {
        let sample_rate = 48000;
        let frequency = 440.0;
        let num_samples = (sample_rate as f32 / 60.0) as usize;

        let mut samples = Vec::with_capacity(num_samples);
        let start_idx = self.audio_buffer.len();

        for i in 0..num_samples {
            let t = (start_idx + i) as f32 / sample_rate as f32;
            let sample = (2.0 * std::f32::consts::PI * frequency * t).sin();
            samples.push(sample);
        }

        self.audio_buffer.extend_from_slice(&samples);

        let samples_needed = self.waveform.calculate_samples_per_screen();
        let max_buffer_size = samples_needed + 200;

        if self.audio_buffer.len() > max_buffer_size {
            let to_remove = self.audio_buffer.len() - max_buffer_size;
            self.audio_buffer.drain(0..to_remove);
        }

        self.waveform.update_samples(self.audio_buffer.clone());
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            // Time/Div controls
            KeyCode::ArrowRight => self.waveform.time_per_division *= 1.1,
            KeyCode::ArrowLeft => self.waveform.time_per_division /= 1.1,

            // Volts/Div controls
            KeyCode::ArrowUp => self.waveform.volts_per_division *= 1.1,
            KeyCode::ArrowDown => self.waveform.volts_per_division /= 1.1,

            // Trigger controls
            KeyCode::KeyT => self.trigger_settings.enabled = !self.trigger_settings.enabled,
            KeyCode::BracketRight => self.trigger_settings.level += 0.1,
            KeyCode::BracketLeft => self.trigger_settings.level -= 0.1,

            _ => {}
        }
    }
}
