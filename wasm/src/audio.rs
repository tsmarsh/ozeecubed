use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioContext, MediaStream, MediaStreamConstraints, ScriptProcessorNode};

pub struct WebAudioCapture {
    _context: AudioContext,
    _stream: MediaStream,
    _processor: ScriptProcessorNode,
    sample_buffer: Rc<RefCell<Vec<f32>>>,
}

impl WebAudioCapture {
    pub async fn new() -> Result<Self, String> {
        let window = web_sys::window().ok_or("No window found")?;
        let navigator = window.navigator();

        // Request microphone access
        let constraints = MediaStreamConstraints::new();
        constraints.set_audio(&JsValue::from(true));
        constraints.set_video(&JsValue::from(false));

        let media_promise = navigator
            .media_devices()
            .map_err(|_| "No media devices")?
            .get_user_media_with_constraints(&constraints)
            .map_err(|_| "Failed to get user media")?;

        let media_result = JsFuture::from(media_promise)
            .await
            .map_err(|_| "Failed to await media promise")?;

        let stream: MediaStream = media_result
            .dyn_into()
            .map_err(|_| "Failed to cast to MediaStream")?;

        // Create audio context
        let context = AudioContext::new().map_err(|_| "Failed to create AudioContext")?;

        // Create source from microphone stream
        let source = context
            .create_media_stream_source(&stream)
            .map_err(|_| "Failed to create media stream source")?;

        // Create script processor (deprecated but widely supported)
        let buffer_size = 4096;
        let processor = context
            .create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
                buffer_size,
                1,
                1,
            )
            .map_err(|_| "Failed to create script processor")?;

        // Set up audio processing callback
        let sample_buffer = Rc::new(RefCell::new(Vec::new()));
        let sample_buffer_clone = sample_buffer.clone();

        let onaudioprocess = Closure::wrap(Box::new(move |event: web_sys::AudioProcessingEvent| {
            let input_buffer = event.input_buffer().unwrap();
            let input_data = input_buffer.get_channel_data(0).unwrap();

            let mut buffer = sample_buffer_clone.borrow_mut();
            buffer.extend_from_slice(&input_data);

            // Keep buffer size reasonable
            if buffer.len() > 48000 {
                let len = buffer.len();
                buffer.drain(0..len - 48000);
            }
        }) as Box<dyn FnMut(_)>);

        processor.set_onaudioprocess(Some(onaudioprocess.as_ref().unchecked_ref()));
        onaudioprocess.forget();

        // Connect the audio graph
        source
            .connect_with_audio_node(&processor)
            .map_err(|_| "Failed to connect source to processor")?;
        processor
            .connect_with_audio_node(&context.destination())
            .map_err(|_| "Failed to connect processor to destination")?;

        Ok(Self {
            _context: context,
            _stream: stream,
            _processor: processor,
            sample_buffer,
        })
    }

    pub fn read_samples(&self, max_samples: usize) -> Vec<f32> {
        let mut buffer = self.sample_buffer.borrow_mut();
        let take_count = max_samples.min(buffer.len());
        let samples: Vec<f32> = buffer.drain(0..take_count).collect();
        samples
    }
}
