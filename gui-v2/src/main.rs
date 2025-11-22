use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::WindowId,
};

mod renderer;
mod state;
mod window;

use window::WindowManager;

struct App {
    window_manager: Option<WindowManager>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_manager.is_none() {
            self.window_manager = Some(WindowManager::new(event_loop));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(ref mut window_manager) = self.window_manager else {
            return;
        };

        if !window_manager.handle_window_event(window_id, &event) {
            match event {
                WindowEvent::CloseRequested => {
                    window_manager.close_window(window_id);
                    if window_manager.is_empty() {
                        event_loop.exit();
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    window_manager.resize(window_id, physical_size);
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(key),
                            ..
                        },
                    ..
                } => {
                    window_manager.handle_keyboard(window_id, key);
                }
                WindowEvent::RedrawRequested => {
                    window_manager.update(window_id);
                    match window_manager.render(window_id) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            window_manager.resize(window_id, window_manager.get_size(window_id));
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            event_loop.exit();
                        }
                        Err(e) => eprintln!("Render error: {e:?}"),
                    }
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref window_manager) = self.window_manager {
            window_manager.request_redraw_all();
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window_manager: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
