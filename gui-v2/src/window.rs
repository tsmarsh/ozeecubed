use std::collections::HashMap;
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::{Window, WindowAttributes, WindowId},
};

use crate::renderer::Renderer;
use crate::state::AppState;

pub struct WindowManager {
    windows: HashMap<WindowId, WindowState>,
    app_state: AppState,
}

struct WindowState {
    window: Arc<Window>,
    renderer: Renderer,
}

impl WindowManager {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let mut windows = HashMap::new();
        let app_state = AppState::new();

        // Create main window
        let window_attrs = WindowAttributes::default()
            .with_title("OzeeCubed - Oscilloscope")
            .with_inner_size(PhysicalSize::new(1280, 720));

        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let renderer = pollster::block_on(Renderer::new(Arc::clone(&window)));
        let window_id = window.id();

        windows.insert(window_id, WindowState { window, renderer });

        Self { windows, app_state }
    }

    pub fn handle_window_event(&mut self, _window_id: WindowId, event: &WindowEvent) -> bool {
        // Return true if event was handled, false otherwise
        match event {
            WindowEvent::MouseInput { .. } | WindowEvent::CursorMoved { .. } => {
                // UI event handling will go here
                false
            }
            _ => false,
        }
    }

    pub fn handle_keyboard(&mut self, _window_id: WindowId, key: KeyCode) {
        self.app_state.handle_key(key);
    }

    pub fn close_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn resize(&mut self, window_id: WindowId, new_size: PhysicalSize<u32>) {
        if let Some(window_state) = self.windows.get_mut(&window_id) {
            window_state.renderer.resize(new_size);
        }
    }

    pub fn get_size(&self, window_id: WindowId) -> PhysicalSize<u32> {
        self.windows
            .get(&window_id)
            .map(|ws| ws.window.inner_size())
            .unwrap_or(PhysicalSize::new(800, 600))
    }

    pub fn update(&mut self, _window_id: WindowId) {
        self.app_state.update();
    }

    pub fn render(&mut self, window_id: WindowId) -> Result<(), wgpu::SurfaceError> {
        if let Some(window_state) = self.windows.get_mut(&window_id) {
            window_state.renderer.render(&self.app_state)
        } else {
            Ok(())
        }
    }

    pub fn request_redraw_all(&self) {
        for window_state in self.windows.values() {
            window_state.window.request_redraw();
        }
    }
}
