mod audio;
mod oscilloscope;
mod ui;

use audio::AudioCapture;
use iced::widget::{column, container};
use iced::{Element, Length, Subscription, Task, Theme};
use oscilloscope::{TriggerSettings, WaveformData};
use std::sync::{Arc, Mutex};
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
    #[allow(dead_code)]
    audio_capture: Option<AudioCapture>,
    #[allow(dead_code)]
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    last_update: Instant,
}

#[derive(Debug, Clone)]
enum Message {
    AudioUpdate,
    Control(ControlMessage),
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
                audio_buffer: Arc::new(Mutex::new(Vec::new())),
                last_update: Instant::now(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AudioUpdate => {
                // For now, generate a test signal
                // TODO: Connect to actual audio capture
                self.generate_test_signal();
                self.canvas.clear_cache();
            }
            Message::Control(control) => {
                self.handle_control(control);
                self.canvas.clear_cache();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let canvas = self.canvas.view(self.waveform.clone());

        let controls = build_controls(
            self.waveform.time_per_division,
            self.waveform.volts_per_division,
            self.trigger_settings.enabled,
            self.trigger_settings.level,
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
        // Update at ~60 FPS
        iced::time::every(Duration::from_millis(16)).map(|_instant| Message::AudioUpdate)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl OzScope {
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
        }
    }

    fn generate_test_signal(&mut self) {
        // Generate a test sine wave for now
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
