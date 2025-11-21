use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{
    HtmlCanvasElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext as GL, WebGlShader,
};

const VERTEX_SHADER: &str = r#"
attribute vec2 position;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
precision mediump float;
uniform vec4 color;
void main() {
    gl_FragColor = color;
}
"#;

pub struct WebGLRenderer {
    gl: GL,
    program: WebGlProgram,
    position_buffer: WebGlBuffer,
    grid_buffer: WebGlBuffer,
}

impl WebGLRenderer {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("Canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;

        let gl = canvas
            .get_context("webgl")?
            .ok_or("No WebGL context")?
            .dyn_into::<GL>()?;

        // Compile shaders
        let vertex_shader = compile_shader(&gl, GL::VERTEX_SHADER, VERTEX_SHADER)?;
        let fragment_shader = compile_shader(&gl, GL::FRAGMENT_SHADER, FRAGMENT_SHADER)?;

        // Link program
        let program = link_program(&gl, &vertex_shader, &fragment_shader)?;
        gl.use_program(Some(&program));

        // Create buffers
        let position_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        let grid_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;

        // Set up grid
        let mut grid_vertices = Vec::new();

        // Vertical grid lines (10 divisions)
        for i in 0..=10 {
            let x = (i as f32 / 10.0) * 2.0 - 1.0;
            grid_vertices.push(x);
            grid_vertices.push(-1.0);
            grid_vertices.push(x);
            grid_vertices.push(1.0);
        }

        // Horizontal grid lines (8 divisions)
        for i in 0..=8 {
            let y = (i as f32 / 8.0) * 2.0 - 1.0;
            grid_vertices.push(-1.0);
            grid_vertices.push(y);
            grid_vertices.push(1.0);
            grid_vertices.push(y);
        }

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&grid_buffer));
        unsafe {
            let grid_array = js_sys::Float32Array::view(&grid_vertices);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &grid_array, GL::STATIC_DRAW);
        }

        Ok(Self {
            gl,
            program,
            position_buffer,
            grid_buffer,
        })
    }

    pub fn render(&self, points: &[(f32, f32)]) {
        let gl = &self.gl;

        // Clear
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);

        gl.use_program(Some(&self.program));

        let position_location = gl.get_attrib_location(&self.program, "position") as u32;
        let color_location = gl.get_uniform_location(&self.program, "color");

        // Draw grid
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.grid_buffer));
        gl.vertex_attrib_pointer_with_i32(position_location, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(position_location);
        gl.uniform4f(color_location.as_ref(), 0.0, 1.0, 0.0, 0.15);
        gl.line_width(1.0);
        gl.draw_arrays(GL::LINES, 0, (10 + 1 + 8 + 1) * 2);

        // Draw center lines brighter
        gl.uniform4f(color_location.as_ref(), 0.0, 1.0, 0.0, 0.5);
        gl.line_width(2.0);
        // Vertical center
        gl.draw_arrays(GL::LINES, 10 * 2, 2);
        // Horizontal center
        gl.draw_arrays(GL::LINES, (10 + 1) * 2 + 8, 2);

        // Draw waveform
        if !points.is_empty() {
            let mut vertices = Vec::new();
            for (x_norm, y_norm) in points {
                let x = x_norm * 2.0 - 1.0;
                let y = -y_norm / 4.0; // Scale for 8 divisions
                vertices.push(x);
                vertices.push(y);
            }

            gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.position_buffer));
            unsafe {
                let vert_array = js_sys::Float32Array::view(&vertices);
                gl.buffer_data_with_array_buffer_view(
                    GL::ARRAY_BUFFER,
                    &vert_array,
                    GL::DYNAMIC_DRAW,
                );
            }

            gl.vertex_attrib_pointer_with_i32(position_location, 2, GL::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(position_location);
            gl.uniform4f(color_location.as_ref(), 0.0, 1.0, 0.0, 1.0);
            gl.line_width(2.0);
            gl.draw_arrays(GL::LINE_STRIP, 0, (vertices.len() / 2) as i32);
        }
    }
}

fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or("Unable to create shader")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error".into()))
    }
}

fn link_program(
    gl: &GL,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl.create_program().ok_or("Unable to create program")?;
    gl.attach_shader(&program, vertex_shader);
    gl.attach_shader(&program, fragment_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error".into()))
    }
}
