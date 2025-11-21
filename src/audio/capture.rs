use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use ringbuf::{traits::*, HeapRb};
use std::sync::{Arc, Mutex};

const BUFFER_SIZE: usize = 48000; // 1 second at 48kHz

pub struct AudioCapture {
    _stream: Stream,
    _buffer: Arc<Mutex<ringbuf::HeapProd<f32>>>,
    _sample_rate: u32,
}

impl AudioCapture {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .ok_or_else(|| "No input device available".to_string())?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {e}"))?;

        let sample_rate = config.sample_rate().0;

        println!(
            "Using audio device: {}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );
        println!("Sample rate: {sample_rate} Hz");
        println!("Channels: {}", config.channels());

        let ring_buffer = HeapRb::<f32>::new(BUFFER_SIZE);
        let (producer, _consumer) = ring_buffer.split();

        let producer = Arc::new(Mutex::new(producer));
        let producer_clone = Arc::clone(&producer);

        let channels = config.channels();
        let stream = Self::build_input_stream(&device, &config.into(), producer_clone, channels)?;
        stream
            .play()
            .map_err(|e| format!("Failed to play stream: {e}"))?;

        Ok(AudioCapture {
            _stream: stream,
            _buffer: producer,
            _sample_rate: sample_rate,
        })
    }

    fn build_input_stream(
        device: &Device,
        config: &StreamConfig,
        producer: Arc<Mutex<ringbuf::HeapProd<f32>>>,
        channels: u16,
    ) -> Result<Stream, String> {
        let err_fn = |err| eprintln!("Audio stream error: {err}");

        let stream = device
            .build_input_stream(
                config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut prod) = producer.lock() {
                        // Mix down to mono by averaging channels
                        for chunk in data.chunks(channels as usize) {
                            let sample = chunk.iter().sum::<f32>() / chunk.len() as f32;
                            let _ = prod.try_push(sample);
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| format!("Failed to build input stream: {e}"))?;

        Ok(stream)
    }
}
