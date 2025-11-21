mod audio;
mod oscilloscope;
mod ui;

use audio::AudioCapture;
use iced::keyboard::{self, Key};
use iced::widget::{column, container};
use iced::{Element, Event, Length, Subscription, Task, Theme};
use oscilloscope::{TriggerSettings, WaveformData};
use std::time::{Duration, Instant};
use ui::controls::{build_controls, ControlMessage};
use ui::WaveformCanvas;

fn main() -> iced::Result {
    iced::application("OzeeCubed", OzScope::update, OzScope::view)
        .subscription(OzScope::subscription)
        .theme(OzScope::theme)
        .run_with(OzScope::new)
}

struct OzScope {
    waveform: WaveformData,
    trigger_settings: TriggerSettings,
    canvas: WaveformCanvas,
    audio_capture: Option<AudioCapture>,
    audio_buffer: Vec<f32>,
    last_update: Instant,
}

#[derive(Debug, Clone)]
enum Message {
    AudioUpdate,
    Control(ControlMessage),
    EventOccurred(Event),
}

impl OzScope {
    fn new() -> (Self, Task<Message>) {
        let sample_rate = 48000;

        let audio_capture = match AudioCapture::new() {
            Ok(capture) => {
                println!("Audio capture initialized successfully");
                Some(capture)
            }
            Err(e) => {
                eprintln!("Failed to initialize audio capture: {e}");
                None
            }
        };

        (
            OzScope {
                waveform: WaveformData::new(sample_rate),
                trigger_settings: TriggerSettings::default(),
                canvas: WaveformCanvas::new(),
                audio_capture,
                audio_buffer: Vec::new(),
                last_update: Instant::now(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AudioUpdate => {
                self.update_audio();
                // Add current waveform to history for persistence effect
                let trigger_settings = TriggerSettings::default();
                let points = self.waveform.get_display_samples(&trigger_settings);
                self.canvas.add_to_history(points);
                self.canvas.clear_cache();
            }
            Message::Control(control) => {
                self.handle_control(control);
                self.canvas.clear_cache();
            }
            Message::EventOccurred(event) => {
                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key, modifiers: _, ..
                }) = event
                {
                    if let Some(control) = Self::key_to_control(&key) {
                        self.handle_control(control);
                        self.canvas.clear_cache();
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let canvas = self.canvas.view(self.waveform.clone());

        let frequency = self.waveform.calculate_frequency();

        let controls = build_controls(
            self.waveform.time_per_division,
            self.waveform.volts_per_division,
            self.trigger_settings.enabled,
            self.trigger_settings.level,
            frequency,
            self.canvas.is_persistence_enabled(),
            self.canvas.get_persistence_frames(),
        )
        .map(Message::Control);

        let content = column![canvas, controls]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            // Update at ~60 FPS
            iced::time::every(Duration::from_millis(16)).map(|_instant| Message::AudioUpdate),
            // Listen for keyboard events
            iced::event::listen().map(Message::EventOccurred),
        ])
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl OzScope {
    fn key_to_control(key: &Key) -> Option<ControlMessage> {
        match key {
            // Time scale controls
            Key::Character(c) if c.as_str() == "+" || c.as_str() == "=" => {
                Some(ControlMessage::IncreaseTimeScale)
            }
            Key::Character(c) if c.as_str() == "-" => Some(ControlMessage::DecreaseTimeScale),
            // Voltage scale controls
            Key::Named(keyboard::key::Named::ArrowUp) => Some(ControlMessage::IncreaseVoltageScale),
            Key::Named(keyboard::key::Named::ArrowDown) => {
                Some(ControlMessage::DecreaseVoltageScale)
            }
            // Trigger controls
            Key::Character(c) if c.as_str() == "t" || c.as_str() == "T" => {
                Some(ControlMessage::ToggleTrigger)
            }
            Key::Character(c) if c.as_str() == "e" || c.as_str() == "E" => {
                Some(ControlMessage::ToggleTriggerEdge)
            }
            Key::Named(keyboard::key::Named::ArrowRight) => {
                Some(ControlMessage::IncreaseTriggerLevel)
            }
            Key::Named(keyboard::key::Named::ArrowLeft) => {
                Some(ControlMessage::DecreaseTriggerLevel)
            }
            // Persistence controls
            Key::Character(c) if c.as_str() == "p" || c.as_str() == "P" => {
                Some(ControlMessage::TogglePersistence)
            }
            _ => None,
        }
    }

    fn handle_control(&mut self, control: ControlMessage) {
        match control {
            ControlMessage::IncreaseTimeScale => {
                self.waveform.increase_time_scale();
            }
            ControlMessage::DecreaseTimeScale => {
                self.waveform.decrease_time_scale();
            }
            ControlMessage::IncreaseVoltageScale => {
                self.waveform.increase_voltage_scale();
            }
            ControlMessage::DecreaseVoltageScale => {
                self.waveform.decrease_voltage_scale();
            }
            ControlMessage::ToggleTrigger => {
                self.trigger_settings.toggle_enabled();
            }
            ControlMessage::ToggleTriggerEdge => {
                self.trigger_settings.toggle_edge();
            }
            ControlMessage::IncreaseTriggerLevel => {
                self.trigger_settings
                    .set_level(self.trigger_settings.level + 0.1);
            }
            ControlMessage::DecreaseTriggerLevel => {
                self.trigger_settings
                    .set_level(self.trigger_settings.level - 0.1);
            }
            ControlMessage::TogglePersistence => {
                self.canvas.toggle_persistence();
            }
            ControlMessage::IncreasePersistence => {
                let current = self.canvas.get_persistence_frames();
                self.canvas.set_persistence_frames(current + 1);
            }
            ControlMessage::DecreasePersistence => {
                let current = self.canvas.get_persistence_frames();
                self.canvas.set_persistence_frames(current.saturating_sub(1));
            }
        }
    }

    fn update_audio(&mut self) {
        if let Some(ref audio_capture) = self.audio_capture {
            // Read new samples from audio capture
            let new_samples = audio_capture.read_samples(4800); // ~100ms at 48kHz

            if !new_samples.is_empty() {
                // Append new samples to buffer
                self.audio_buffer.extend_from_slice(&new_samples);

                // Keep buffer at a reasonable size (1 second of audio)
                let max_buffer_size = audio_capture.sample_rate as usize;
                if self.audio_buffer.len() > max_buffer_size {
                    let excess = self.audio_buffer.len() - max_buffer_size;
                    self.audio_buffer.drain(0..excess);
                }

                // Update waveform with current buffer
                self.waveform.update_samples(self.audio_buffer.clone());
            }
        } else {
            // Fallback: generate test signal if no audio capture
            self.generate_test_signal();
        }
    }

    fn generate_test_signal(&mut self) {
        // Generate a test sine wave as fallback
        let sample_rate = self.waveform.sample_rate as f32;
        let frequency = 440.0; // A4 note
        let duration = 0.1; // 100ms of samples

        let num_samples = (sample_rate * duration) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        let time_offset = self.last_update.elapsed().as_secs_f32();

        for i in 0..num_samples {
            let t = (i as f32 / sample_rate) + time_offset;
            let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
            samples.push(sample);
        }

        self.waveform.update_samples(samples);
    }
}
