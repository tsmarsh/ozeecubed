use iced::mouse;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

pub struct SpectrumCanvas {
    cache: Cache,
    spectrum: Vec<f32>,
    sample_rate: u32,
}

impl SpectrumCanvas {
    pub fn new() -> Self {
        Self {
            cache: Cache::new(),
            spectrum: Vec::new(),
            sample_rate: 48000,
        }
    }

    pub fn update_spectrum(&mut self, samples: &[f32], sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.spectrum = self.compute_spectrum(samples);
        self.cache.clear();
    }

    pub fn view<'a>(&'a self) -> iced::Element<'a, ()> {
        iced::widget::canvas(self as &'a Self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    fn compute_spectrum(&self, samples: &[f32]) -> Vec<f32> {
        if samples.is_empty() {
            return vec![];
        }

        // Use a power of 2 for FFT efficiency
        let fft_size = samples.len().next_power_of_two().min(4096);
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .take(fft_size)
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        // Pad with zeros if needed
        buffer.resize(fft_size, Complex::new(0.0, 0.0));

        // Apply Hann window to reduce spectral leakage
        for (i, sample) in buffer.iter_mut().enumerate() {
            let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / fft_size as f32).cos());
            *sample = *sample * window;
        }

        // Perform FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        fft.process(&mut buffer);

        // Compute magnitude spectrum (only first half - positive frequencies)
        // Convert to dB scale
        buffer
            .iter()
            .take(fft_size / 2)
            .map(|c| {
                let magnitude = c.norm() / (fft_size as f32).sqrt();
                // Convert to dB, with floor at -80 dB
                20.0 * (magnitude.max(0.00001)).log10()
            })
            .collect()
    }
}

impl canvas::Program<()> for SpectrumCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let width = frame.width();
            let height = frame.height();

            // Draw dark background
            frame.fill_rectangle(
                Point::ORIGIN,
                Size::new(width, height),
                Color::from_rgb8(10, 10, 10),
            );

            // Draw grid
            draw_spectrum_grid(frame, width, height);

            // Draw spectrum
            if !self.spectrum.is_empty() {
                draw_spectrum(frame, width, height, &self.spectrum);
            }
        });

        vec![geometry]
    }
}

fn draw_spectrum_grid(frame: &mut Frame, width: f32, height: f32) {
    let grid_color = Color::from_rgba8(0, 255, 65, 0.15);
    let center_color = Color::from_rgba8(0, 255, 65, 0.3);

    // Vertical lines (frequency divisions)
    let num_v_divs = 10;
    for i in 0..=num_v_divs {
        let x = (i as f32 / num_v_divs as f32) * width;
        let color = if i == 0 { center_color } else { grid_color };

        let line = Path::line(Point::new(x, 0.0), Point::new(x, height));
        frame.stroke(&line, Stroke::default().with_color(color).with_width(1.0));
    }

    // Horizontal lines (amplitude divisions - dB scale)
    let num_h_divs = 8;
    for i in 0..=num_h_divs {
        let y = (i as f32 / num_h_divs as f32) * height;
        let color = if i == num_h_divs {
            center_color // Bottom line (-80 dB)
        } else {
            grid_color
        };

        let line = Path::line(Point::new(0.0, y), Point::new(width, y));
        frame.stroke(&line, Stroke::default().with_color(color).with_width(1.0));
    }

    // Draw frequency labels at bottom
    let label_color = Color::from_rgba8(0, 255, 65, 0.7);
    for i in 0..=5 {
        let x = (i as f32 / 5.0) * width;
        let freq_khz = (i as f32 / 5.0) * 24.0; // 0-24 kHz for 48kHz sample rate
        let label = if freq_khz == 0.0 {
            "0".to_string()
        } else {
            format!("{:.0}k", freq_khz)
        };

        frame.fill_text(Text {
            content: label,
            position: Point::new(x + 5.0, height - 18.0),
            color: label_color,
            size: 11.0.into(),
            ..Default::default()
        });
    }

    // Draw dB labels on left
    for i in 0..=4 {
        let y = (i as f32 / 4.0) * height;
        let db = -(80.0 - (i as f32 / 4.0) * 80.0); // -80 dB to 0 dB
        let label = format!("{:.0}", db);

        frame.fill_text(Text {
            content: label,
            position: Point::new(5.0, y + 5.0),
            color: label_color,
            size: 11.0.into(),
            ..Default::default()
        });
    }
}

fn draw_spectrum(frame: &mut Frame, width: f32, height: f32, spectrum: &[f32]) {
    if spectrum.len() < 2 {
        return;
    }

    let waveform_color = Color::from_rgb8(0, 255, 65);

    // Build path for spectrum curve
    let mut path_builder = canvas::path::Builder::new();

    // Map dB range: -80 to 0 dB
    let db_min = -80.0;
    let db_max = 0.0;

    for (i, &db) in spectrum.iter().enumerate() {
        let x = (i as f32 / spectrum.len() as f32) * width;
        // Normalize dB to 0-1 range, then invert for screen coordinates
        let normalized = (db - db_min) / (db_max - db_min);
        let y = height * (1.0 - normalized.clamp(0.0, 1.0));

        if i == 0 {
            path_builder.move_to(Point::new(x, y));
        } else {
            path_builder.line_to(Point::new(x, y));
        }
    }

    let path = path_builder.build();
    frame.stroke(&path, Stroke::default().with_color(waveform_color).with_width(2.0));
}
