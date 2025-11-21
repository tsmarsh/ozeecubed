pub mod controls;

use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path, Program, Stroke};
use iced::mouse;
use iced::{Color, Point, Rectangle, Size, Theme};

use crate::oscilloscope::WaveformData;

pub const SCOPE_GREEN: Color = Color::from_rgb(0.0, 1.0, 0.0);
pub const GRID_GREEN: Color = Color::from_rgba(0.0, 1.0, 0.0, 0.3);
pub const BACKGROUND: Color = Color::BLACK;

pub struct WaveformCanvas {
    cache: Cache,
}

impl WaveformCanvas {
    pub fn new() -> Self {
        WaveformCanvas {
            cache: Cache::default(),
        }
    }

    pub fn view<'a, Message>(&'a self, waveform: WaveformData) -> Canvas<WaveformData, Message> {
        Canvas::new(waveform)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl<Message> Program<Message> for WaveformData {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Draw background
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), BACKGROUND);

        // Draw grid
        draw_grid(&mut frame, bounds.size());

        // Draw waveform
        draw_waveform(&mut frame, bounds.size(), self);

        vec![frame.into_geometry()]
    }
}

fn draw_grid(frame: &mut Frame, size: Size) {
    let divisions_x = 10;
    let divisions_y = 8;

    let width = size.width;
    let height = size.height;

    // Draw vertical lines
    for i in 0..=divisions_x {
        let x = (i as f32 / divisions_x as f32) * width;
        let path = Path::line(Point::new(x, 0.0), Point::new(x, height));
        frame.stroke(
            &path,
            Stroke::default().with_color(GRID_GREEN).with_width(1.0),
        );
    }

    // Draw horizontal lines
    for i in 0..=divisions_y {
        let y = (i as f32 / divisions_y as f32) * height;
        let path = Path::line(Point::new(0.0, y), Point::new(width, y));
        frame.stroke(
            &path,
            Stroke::default().with_color(GRID_GREEN).with_width(1.0),
        );
    }

    // Draw center lines brighter
    let center_x = width / 2.0;
    let center_y = height / 2.0;

    let path = Path::line(Point::new(center_x, 0.0), Point::new(center_x, height));
    frame.stroke(
        &path,
        Stroke::default()
            .with_color(Color::from_rgba(0.0, 1.0, 0.0, 0.5))
            .with_width(2.0),
    );

    let path = Path::line(Point::new(0.0, center_y), Point::new(width, center_y));
    frame.stroke(
        &path,
        Stroke::default()
            .with_color(Color::from_rgba(0.0, 1.0, 0.0, 0.5))
            .with_width(2.0),
    );
}

fn draw_waveform(frame: &mut Frame, size: Size, waveform: &WaveformData) {
    if waveform.samples.is_empty() {
        return;
    }

    let width = size.width;
    let height = size.height;
    let center_y = height / 2.0;

    // Get display samples (normalized)
    let trigger_settings = crate::oscilloscope::TriggerSettings::default();
    let points = waveform.get_display_samples(&trigger_settings);

    if points.is_empty() {
        return;
    }

    let mut path_builder = canvas::path::Builder::new();

    // Convert normalized coordinates to screen coordinates
    for (i, &(x_norm, y_norm)) in points.iter().enumerate() {
        let x = x_norm * width;
        let y = center_y - (y_norm * height / 8.0); // 8 vertical divisions

        if i == 0 {
            path_builder.move_to(Point::new(x, y));
        } else {
            path_builder.line_to(Point::new(x, y));
        }
    }

    let path = path_builder.build();
    frame.stroke(
        &path,
        Stroke::default().with_color(SCOPE_GREEN).with_width(2.0),
    );
}
