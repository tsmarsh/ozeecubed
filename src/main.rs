mod audio;
mod oscilloscope;
mod ui;

use audio::AudioCapture;
use iced::keyboard::{self, Key};
use iced::widget::{column, container, row};
use iced::{Element, Event, Length, Subscription, Task, Theme};
use oscilloscope::{TriggerSettings, WaveformData};
use std::time::Duration;
use ui::controls::{build_controls, ControlMessage, LayoutMode, Measurements};
use ui::{SpectrumCanvas, WaveformCanvas};

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
    spectrum_canvas: SpectrumCanvas,
    audio_capture: Option<AudioCapture>,
    audio_buffer: Vec<f32>,
    layout_mode: LayoutMode,
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
                spectrum_canvas: SpectrumCanvas::new(),
                audio_capture,
                audio_buffer: Vec::new(),
                layout_mode: LayoutMode::SideBySide,
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
        let scope_canvas = self.canvas.view(self.waveform.clone());
        let spectrum_canvas = self.spectrum_canvas.view().map(|_| Message::AudioUpdate);

        let measurements = Measurements {
            frequency: self.waveform.calculate_frequency(),
            peak_to_peak: self.waveform.calculate_peak_to_peak(),
            rms: self.waveform.calculate_rms(),
            duty_cycle: self.waveform.calculate_duty_cycle(),
        };

        let controls = build_controls(
            self.waveform.time_per_division,
            self.waveform.volts_per_division,
            self.trigger_settings.enabled,
            self.trigger_settings.level,
            &measurements,
            self.canvas.is_persistence_enabled(),
            self.canvas.get_persistence_frames(),
            self.layout_mode,
        )
        .map(Message::Control);

        // Choose layout based on mode
        let canvas_view: Element<'_, Message> = match self.layout_mode {
            LayoutMode::ScopeOnly => column![scope_canvas].into(),
            LayoutMode::SpectrumOnly => column![spectrum_canvas].into(),
            LayoutMode::SideBySide => row![scope_canvas, spectrum_canvas]
                .spacing(2)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            LayoutMode::Stacked => column![scope_canvas, spectrum_canvas]
                .spacing(2)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        };

        let content = column![canvas_view, controls]
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
            ControlMessage::SetTimeScale(value) => {
                self.waveform.time_per_division = value;
            }
            ControlMessage::IncreaseVoltageScale => {
                self.waveform.increase_voltage_scale();
            }
            ControlMessage::DecreaseVoltageScale => {
                self.waveform.decrease_voltage_scale();
            }
            ControlMessage::SetVoltageScale(value) => {
                self.waveform.volts_per_division = value;
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
            ControlMessage::SetTriggerLevel(value) => {
                self.trigger_settings.set_level(value);
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
                self.canvas
                    .set_persistence_frames(current.saturating_sub(1));
            }
            ControlMessage::SetPersistenceFrames(value) => {
                self.canvas.set_persistence_frames(value as usize);
            }
            ControlMessage::SetLayoutMode(mode) => {
                self.layout_mode = mode;
            }
        }
    }

    fn update_audio(&mut self) {
        if let Some(ref audio_capture) = self.audio_capture {
            // Read ALL available samples for minimal latency
            let new_samples = audio_capture.read_samples(usize::MAX);

            if !new_samples.is_empty() {
                // Near-zero-latency mode: keep only what we need for one stable screen
                // Trigger detection happens on current data, not historical accumulation
                self.audio_buffer.extend_from_slice(&new_samples);

                // Keep exactly what we need: 1 screen + small margin for trigger search
                let samples_needed = self.waveform.calculate_samples_per_screen();
                let max_buffer_size = samples_needed + 200; // 200 samples = ~4ms margin

                if self.audio_buffer.len() > max_buffer_size {
                    let to_remove = self.audio_buffer.len() - max_buffer_size;
                    self.audio_buffer.drain(0..to_remove);
                }

                // Update waveform with current buffer
                self.waveform.update_samples(self.audio_buffer.clone());

                // Update spectrum analyzer
                self.spectrum_canvas.update_spectrum(&self.audio_buffer, self.waveform.sample_rate);
            }
        } else {
            // Fallback: generate test signal if no audio capture
            self.generate_test_signal();
        }
    }

    fn generate_test_signal(&mut self) {
        // Generate a test sine wave as fallback when no audio device is available
        let sample_rate = self.waveform.sample_rate as f32;
        let frequency = 440.0; // A4 note
        let duration = 0.1; // 100ms of samples

        let num_samples = (sample_rate * duration) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        // Use audio_buffer length as a simple phase accumulator
        let phase_offset = self.audio_buffer.len() as f32 / sample_rate;

        for i in 0..num_samples {
            let t = (i as f32 / sample_rate) + phase_offset;
            let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
            samples.push(sample);
        }

        self.waveform.update_samples(samples);
    }
}
