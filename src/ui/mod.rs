pub mod controls;
pub mod spectrum;

use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path, Program, Stroke};
use iced::{Color, Point, Rectangle, Size, Theme};
use std::collections::VecDeque;

use crate::oscilloscope::WaveformData;
pub use spectrum::SpectrumCanvas;

const GRID_GREEN: Color = Color::from_rgba(0.0, 1.0, 0.0, 0.3);
const BACKGROUND: Color = Color::BLACK;

pub struct WaveformCanvas {
    cache: Cache,
    history: VecDeque<Vec<(f32, f32)>>,
    persistence_enabled: bool,
    persistence_frames: usize,
}

impl Default for WaveformCanvas {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WaveformWithHistory {
    pub waveform: WaveformData,
    pub history: VecDeque<Vec<(f32, f32)>>,
    pub persistence_enabled: bool,
}

impl WaveformCanvas {
    pub fn new() -> Self {
        WaveformCanvas {
            cache: Cache::default(),
            history: VecDeque::new(),
            persistence_enabled: true,
            persistence_frames: 10,
        }
    }

    pub fn view<Message>(&self, waveform: WaveformData) -> Canvas<WaveformWithHistory, Message> {
        let data = WaveformWithHistory {
            waveform,
            history: self.history.clone(),
            persistence_enabled: self.persistence_enabled,
        };
        Canvas::new(data)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn add_to_history(&mut self, points: Vec<(f32, f32)>) {
        if self.persistence_enabled && !points.is_empty() {
            self.history.push_back(points);

            // Keep only the configured number of frames
            while self.history.len() > self.persistence_frames {
                self.history.pop_front();
            }
        }
    }

    pub fn toggle_persistence(&mut self) {
        self.persistence_enabled = !self.persistence_enabled;
        if !self.persistence_enabled {
            self.history.clear();
        }
    }

    pub fn set_persistence_frames(&mut self, frames: usize) {
        self.persistence_frames = frames.clamp(1, 30);
        // Trim history if new limit is smaller
        while self.history.len() > self.persistence_frames {
            self.history.pop_front();
        }
    }

    pub fn is_persistence_enabled(&self) -> bool {
        self.persistence_enabled
    }

    pub fn get_persistence_frames(&self) -> usize {
        self.persistence_frames
    }

    #[cfg(test)]
    pub fn get_history(&self) -> &VecDeque<Vec<(f32, f32)>> {
        &self.history
    }
}

impl<Message> Program<Message> for WaveformWithHistory {
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

        // Draw historical waveforms with fading alpha
        if self.persistence_enabled {
            let history_count = self.history.len();
            for (i, points) in self.history.iter().enumerate() {
                // Calculate alpha based on age (older = more transparent)
                let age_factor = (i + 1) as f32 / (history_count + 1) as f32;
                let alpha = age_factor * 0.6; // Max 60% opacity for history
                draw_waveform_points(&mut frame, bounds.size(), points, alpha);
            }
        }

        // Draw current waveform (full brightness)
        draw_waveform(&mut frame, bounds.size(), &self.waveform);

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

    // Get display samples (normalized)
    let trigger_settings = crate::oscilloscope::TriggerSettings::default();
    let points = waveform.get_display_samples(&trigger_settings);

    if points.is_empty() {
        return;
    }

    // Draw with full opacity
    draw_waveform_points(frame, size, &points, 1.0);
}

fn draw_waveform_points(frame: &mut Frame, size: Size, points: &[(f32, f32)], alpha: f32) {
    if points.is_empty() {
        return;
    }

    let width = size.width;
    let height = size.height;
    let center_y = height / 2.0;

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
    let color = Color::from_rgba(0.0, 1.0, 0.0, alpha);
    frame.stroke(&path, Stroke::default().with_color(color).with_width(2.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_new() {
        let canvas = WaveformCanvas::new();
        assert!(canvas.is_persistence_enabled());
        assert_eq!(canvas.get_persistence_frames(), 10);
        assert_eq!(canvas.get_history().len(), 0);
    }

    #[test]
    fn test_toggle_persistence() {
        let mut canvas = WaveformCanvas::new();
        assert!(canvas.is_persistence_enabled());

        canvas.toggle_persistence();
        assert!(!canvas.is_persistence_enabled());

        canvas.toggle_persistence();
        assert!(canvas.is_persistence_enabled());
    }

    #[test]
    fn test_toggle_persistence_clears_history() {
        let mut canvas = WaveformCanvas::new();
        let points = vec![(0.0, 0.5), (1.0, 0.5)];

        canvas.add_to_history(points.clone());
        assert_eq!(canvas.get_history().len(), 1);

        canvas.toggle_persistence(); // Turn off
        assert_eq!(canvas.get_history().len(), 0);
    }

    #[test]
    fn test_add_to_history() {
        let mut canvas = WaveformCanvas::new();
        let points1 = vec![(0.0, 0.5), (1.0, 0.5)];
        let points2 = vec![(0.0, 0.3), (1.0, 0.3)];

        canvas.add_to_history(points1);
        assert_eq!(canvas.get_history().len(), 1);

        canvas.add_to_history(points2);
        assert_eq!(canvas.get_history().len(), 2);
    }

    #[test]
    fn test_add_to_history_when_disabled() {
        let mut canvas = WaveformCanvas::new();
        canvas.toggle_persistence(); // Turn off

        let points = vec![(0.0, 0.5), (1.0, 0.5)];
        canvas.add_to_history(points);

        assert_eq!(canvas.get_history().len(), 0);
    }

    #[test]
    fn test_add_empty_points_to_history() {
        let mut canvas = WaveformCanvas::new();
        let points = vec![];

        canvas.add_to_history(points);
        assert_eq!(canvas.get_history().len(), 0);
    }

    #[test]
    fn test_history_max_size() {
        let mut canvas = WaveformCanvas::new();
        let points = vec![(0.0, 0.5), (1.0, 0.5)];

        // Add more than the maximum
        for _ in 0..15 {
            canvas.add_to_history(points.clone());
        }

        // Should only keep the configured number of frames
        assert_eq!(canvas.get_history().len(), 10);
    }

    #[test]
    fn test_set_persistence_frames() {
        let mut canvas = WaveformCanvas::new();

        canvas.set_persistence_frames(20);
        assert_eq!(canvas.get_persistence_frames(), 20);

        canvas.set_persistence_frames(5);
        assert_eq!(canvas.get_persistence_frames(), 5);
    }

    #[test]
    fn test_set_persistence_frames_clamping() {
        let mut canvas = WaveformCanvas::new();

        // Test minimum
        canvas.set_persistence_frames(0);
        assert_eq!(canvas.get_persistence_frames(), 1);

        // Test maximum
        canvas.set_persistence_frames(100);
        assert_eq!(canvas.get_persistence_frames(), 30);
    }

    #[test]
    fn test_set_persistence_frames_trims_history() {
        let mut canvas = WaveformCanvas::new();
        let points = vec![(0.0, 0.5), (1.0, 0.5)];

        // Add 10 frames
        for _ in 0..10 {
            canvas.add_to_history(points.clone());
        }
        assert_eq!(canvas.get_history().len(), 10);

        // Reduce to 5 frames
        canvas.set_persistence_frames(5);
        assert_eq!(canvas.get_history().len(), 5);
    }

    #[test]
    fn test_history_fifo_order() {
        let mut canvas = WaveformCanvas::new();
        canvas.set_persistence_frames(3);

        let points1 = vec![(0.0, 0.1)];
        let points2 = vec![(0.0, 0.2)];
        let points3 = vec![(0.0, 0.3)];
        let points4 = vec![(0.0, 0.4)];

        canvas.add_to_history(points1);
        canvas.add_to_history(points2.clone());
        canvas.add_to_history(points3.clone());
        canvas.add_to_history(points4.clone());

        // First one should have been popped
        assert_eq!(canvas.get_history().len(), 3);

        // Check that the oldest (points1) was removed
        let history: Vec<_> = canvas.get_history().iter().collect();
        assert_eq!(history[0], &points2);
        assert_eq!(history[1], &points3);
        assert_eq!(history[2], &points4);
    }
}
