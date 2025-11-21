use wasm_bindgen::prelude::*;

use ozeecubed_core::oscilloscope::{TriggerSettings, WaveformData};

mod audio;
mod webgl;

use audio::WebAudioCapture;
use webgl::WebGLRenderer;

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"OzeeCubed WASM initializing...".into());

    Ok(())
}

#[wasm_bindgen]
pub struct OzScopeWasm {
    waveform: WaveformData,
    trigger_settings: TriggerSettings,
    audio_capture: Option<WebAudioCapture>,
    renderer: Option<WebGLRenderer>,
    audio_buffer: Vec<f32>,
}

impl Default for OzScopeWasm {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl OzScopeWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self {
            waveform: WaveformData::new(48000),
            trigger_settings: TriggerSettings::default(),
            audio_capture: None,
            renderer: None,
            audio_buffer: Vec::new(),
        }
    }

    pub async fn init_audio(&mut self) -> Result<(), JsValue> {
        match WebAudioCapture::new().await {
            Ok(capture) => {
                web_sys::console::log_1(&"Audio capture initialized".into());
                self.audio_capture = Some(capture);
                Ok(())
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to init audio: {e}").into());
                Err(JsValue::from_str(&e))
            }
        }
    }

    pub fn init_renderer(&mut self, canvas_id: &str) -> Result<(), JsValue> {
        let renderer = WebGLRenderer::new(canvas_id)?;
        web_sys::console::log_1(&"WebGL renderer initialized".into());
        self.renderer = Some(renderer);
        Ok(())
    }

    pub fn update(&mut self) {
        // Update audio buffer
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
        }
    }

    pub fn render(&self) {
        if let Some(ref renderer) = self.renderer {
            let points = self.waveform.get_display_samples(&self.trigger_settings);
            renderer.render(&points);
        }
    }

    pub fn set_time_per_div(&mut self, value: f32) {
        self.waveform.time_per_division = value;
    }

    pub fn set_volts_per_div(&mut self, value: f32) {
        self.waveform.volts_per_division = value;
    }

    pub fn set_trigger_enabled(&mut self, enabled: bool) {
        self.trigger_settings.enabled = enabled;
    }

    pub fn set_trigger_level(&mut self, level: f32) {
        self.trigger_settings.level = level;
    }
}
