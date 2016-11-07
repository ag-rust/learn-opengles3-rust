extern crate gleam;
extern crate glutin;

use std::ptr;
use std::ffi::CStr;
use std::mem::size_of;
use std::os::raw::c_char;
use gleam::gl;
use gleam::gl::{GLenum, GLfloat, GLint, GLsizei, GLuint, GLushort};

struct Context {
    program_object: GLuint,
    vao_id: GLuint,
}

fn get_string(name: GLenum) -> String {
    unsafe {
        let buffer = gl::GetString(name);
        CStr::from_ptr(buffer as *mut c_char)
            .to_owned()
            .into_string()
            .ok()
            .unwrap()
    }
}

fn get_shaderiv(shader: GLuint, pname: GLenum) -> GLint {
    let mut value = 0;
    unsafe {
        gl::GetShaderiv(shader, pname, &mut value);
        value
    }
}

fn get_programiv(program: GLuint, pname: GLenum) -> GLint {
    let mut value = 0;
    unsafe {
        gl::GetProgramiv(program, pname, &mut value);
        value
    }
}

fn load_shader(shader_type: GLenum, source: &'static [u8]) -> Option<GLuint> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        if shader == 0 {
            return None;
        }
        gl::ShaderSource(shader, 1, [source.as_ptr() as *const _].as_ptr(), ptr::null());
        gl::CompileShader(shader);
        let compiled = get_shaderiv(shader, gl::COMPILE_STATUS);
        if compiled == 0 {
            let info_len = get_shaderiv(shader, gl::INFO_LOG_LENGTH);
            if info_len > 1 {
                let mut buffer = vec![0u8; info_len as usize];
                let mut length = 0;
                gl::GetShaderInfoLog(shader, info_len, &mut length, buffer.as_mut_ptr() as *mut _);
                println!("{}", String::from_utf8(buffer).ok().unwrap());
            }
            gl::DeleteShader(shader);
            return None;
        }
        Some(shader)
    }
}

fn initialize() -> Option<Context> {
    unsafe {
        let vertex_shader = load_shader(gl::VERTEX_SHADER, VS_SRC).unwrap();
        let fragment_shader = load_shader(gl::FRAGMENT_SHADER, FS_SRC).unwrap();
        let program_object = gl::CreateProgram();
        if program_object == 0 {
            return None
        }
        gl::AttachShader(program_object, vertex_shader);
        gl::AttachShader(program_object, fragment_shader);
        gl::BindAttribLocation(program_object, 0, b"vPosition\0".as_ptr() as *const _);
        gl::LinkProgram(program_object);
        let linked = get_programiv(program_object, gl::LINK_STATUS);
        println!("{}", linked);

        let mut vbo_ids = [0u32; 2];
        gl::GenBuffers(2, vbo_ids.as_mut_ptr() as *mut _);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_ids[0]);
        gl::BufferData(gl::ARRAY_BUFFER, (VERTICES.len() * size_of::<GLfloat>()) as isize, VERTICES.as_ptr() as *const _, gl::STATIC_DRAW);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo_ids[1]);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (INDICES.len() * size_of::<GLushort>()) as isize, INDICES.as_ptr() as *const _, gl::STATIC_DRAW);

        let mut vao_ids = [0u32; 1];
        gl::GenVertexArrays(1, vao_ids.as_mut_ptr() as *mut _);
        gl::BindVertexArray(vao_ids[0]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_ids[0]);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo_ids[1]);
        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 28, 0 as *const _);
        gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 28, 12 as *const _);
        gl::BindVertexArray(0);

        gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        Some(Context {
            program_object: program_object,
            vao_id: vao_ids[0],
        })
    }
}

fn draw(context: &Context, width: GLsizei, height: GLsizei) {
    unsafe {
        gl::Viewport(0, 0, width, height);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::UseProgram(context.program_object);
        gl::BindVertexArray(context.vao_id);
        gl::DrawElements(gl::TRIANGLES, 3, gl::UNSIGNED_SHORT, 0 as *const _);
        gl::BindVertexArray(0);
    }
}

fn main() {
    let window = glutin::Window::new().unwrap();

    unsafe {
        let _ = window.make_current();
    }

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    println!("GL VERSION: {}", get_string(gl::VERSION));

    let context = initialize().unwrap();

    for event in window.wait_events() {
        let (width, height) = window.get_inner_size().unwrap();
        draw(&context, width as GLsizei, height as GLsizei);

        let _ = window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => (),
        }
    }
}

const VS_SRC: &'static [u8] = b"
#version 300 es
layout(location = 0) in vec4 a_position;
layout(location = 1) in vec4 a_color;
out vec4 v_color;
void main() {
    v_color = a_color;
    gl_Position = a_position;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 300 es
precision mediump float;
in vec4 v_color;
out vec4 o_fragColor;
void main() {
    o_fragColor = v_color;
}
\0";

const VERTICES: [GLfloat; 21] = [
    0.0, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0,
    -0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0,
    0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 1.0,
];

const INDICES: [GLushort; 3] = [0, 1, 2];
