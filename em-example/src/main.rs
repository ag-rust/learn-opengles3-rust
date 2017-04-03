extern crate libc;
extern crate gleam;
extern crate emscripten_sys;

use gleam::gl;
use gleam::gl::{GLenum, GLuint};
use emscripten_sys::{
    emscripten_set_main_loop_arg,
    emscripten_webgl_init_context_attributes,
    emscripten_webgl_create_context,
    emscripten_webgl_make_context_current,
    emscripten_GetProcAddress,
    EmscriptenWebGLContextAttributes,
};

type GlPtr = std::rc::Rc<gl::Gl>;

#[repr(C)]
struct Context {
    gl: GlPtr,
    program: GLuint,
    buffer: GLuint,
}

fn load_shader(gl: &GlPtr, shader_type: GLenum, source: &[&[u8]]) -> Option<GLuint> {
    let shader = gl.create_shader(shader_type);
    if shader == 0 {
        return None;
    }
    gl.shader_source(shader, source);
    gl.compile_shader(shader);
    let compiled = gl.get_shader_iv(shader, gl::COMPILE_STATUS);
    if compiled == 0 {
        let log = gl.get_shader_info_log(shader);
        println!("{}", log);
        gl.delete_shader(shader);
        return None;
    }
    Some(shader)
}

fn init_buffer(gl: &GlPtr) -> Option<GLuint> {
    let v_vertices: [f32; 9] = [
        0.0, 0.5, 0.0,
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
    ];
    let buffers = gl.gen_buffers(1);
    gl.bind_buffer(gl::ARRAY_BUFFER, buffers[0]);
    gl.buffer_data_untyped(gl::ARRAY_BUFFER, 36, v_vertices.as_ptr() as *const _, gl::STATIC_DRAW);
    Some(buffers[0])
}

impl Context {
    fn new(gl: GlPtr) -> Context {
        let v_shader = load_shader(&gl, gl::VERTEX_SHADER, VS_SRC).unwrap();
        let f_shader = load_shader(&gl, gl::FRAGMENT_SHADER, FS_SRC).unwrap();
        let program = gl.create_program();
        gl.attach_shader(program, v_shader);
        gl.attach_shader(program, f_shader);
        gl.link_program(program);
        gl.use_program(program);
        gl.enable_vertex_attrib_array(0);
        let buffer = init_buffer(&gl).unwrap();
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        Context {
            gl: gl,
            program: program,
            buffer: buffer,
        }
    }

    fn draw(&self) {
        let gl = &self.gl;
        gl.viewport(0, 0, 500, 500);
        gl.clear(gl::COLOR_BUFFER_BIT);
        gl.use_program(self.program);
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buffer);
        gl.vertex_attrib_pointer(0, 3, gl::FLOAT, false, 0, 0);
        gl.draw_arrays(gl::TRIANGLES, 0, 3);
    }
}

fn step(ctx: &mut Context) {
    ctx.draw();
}

extern fn loop_wrapper(ctx: *mut libc::c_void) {
    unsafe {
        let mut ctx = &mut *(ctx as *mut Context);
        step(&mut ctx);
    }
}

fn main() {
    unsafe {
        let mut attributes: EmscriptenWebGLContextAttributes = std::mem::uninitialized();
        emscripten_webgl_init_context_attributes(&mut attributes);
        attributes.majorVersion = 2;
        let handle = emscripten_webgl_create_context(std::ptr::null(), &attributes);
        emscripten_webgl_make_context_current(handle);
        let gl = gl::GlesFns::load_with(|addr| {
            let addr = std::ffi::CString::new(addr).unwrap();
            emscripten_GetProcAddress(addr.into_raw() as *const _) as *const _
        });
        let mut ctx = Context::new(gl);
        let ptr = &mut ctx as *mut _ as *mut libc::c_void;
        emscripten_set_main_loop_arg(Some(loop_wrapper), ptr, 0, 1);
    }
}

const VS_SRC: &'static [&[u8]] = &[b"#version 300 es
layout(location = 0) in vec4 vPosition;
void main() {
    gl_Position = vPosition;
}"];

const FS_SRC: &'static [&[u8]] = &[b"#version 300 es
precision mediump float;
out vec4 fragColor;
void main() {
    fragColor = vec4(1.0, 0.0, 0.0, 1.0);
}"];
