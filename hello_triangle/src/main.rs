extern crate gleam;
extern crate glutin;

use std::ptr;
use std::ffi::CStr;
use std::os::raw::c_char;
use gleam::gl;
use gleam::gl::{GLenum, GLfloat, GLint, GLuint};

struct Context {
    program_object: GLuint,
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
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        Some(Context {
            program_object: program_object,
        })
    }
}

fn draw(context: &Context) {
    let width = 500;
    let height = 500;

    let v_vertices: [f32; 9] = [
        0.0, 0.5, 0.0,
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
    ];

    unsafe {
        // gl::Viewport(0, 0, 1, 1);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::UseProgram(context.program_object);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, v_vertices.as_ptr() as *const _);
        gl::EnableVertexAttribArray(0);
        gl::PointSize(10.0);
        gl::DrawArrays(gl::POINTS, 0, 3);
    }
}

fn main() {
    let window = glutin::Window::new().unwrap();

    unsafe {
        let _ = window.make_current();
    }

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    println!("{}", get_string(gl::VERSION));

    let context = initialize().unwrap();

    for event in window.wait_events() {
        draw(&context);

        let _ = window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => (),
        }
    }
}

const VS_SRC: &'static [u8] = b"
#version 100
attribute vec3 vPosition;
void main() {
    gl_Position = vec4(vPosition, 1.0);
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
void main() {
    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}
\0";
