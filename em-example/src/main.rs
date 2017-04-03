extern crate libc;
extern crate gleam;

mod emscripten;

use gleam::gl;
use gleam::gl::{GLuint};
use emscripten::{
    emscripten_set_main_loop_arg,
    emscripten_webgl_init_context_attributes,
    emscripten_webgl_create_context,
    emscripten_webgl_make_context_current,
    emscripten_GetProcAddress,
    EmscriptenWebGLContextAttributes,
};

#[repr(C)]
struct Context {
    gl: std::rc::Rc<gl::Gl>,
    program: GLuint,
    buffer: GLuint,
}

impl Context {
    fn new(gl: std::rc::Rc<gl::Gl>) -> Context {
        let v_vertices: [f32; 9] = [
            0.0, 0.5, 0.0,
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
        ];

        let v_shader = gl.create_shader(gl::VERTEX_SHADER);
        gl.shader_source(v_shader, VS_SRC);
        gl.compile_shader(v_shader);
        let f_shader = gl.create_shader(gl::FRAGMENT_SHADER);
        gl.shader_source(f_shader, FS_SRC);
        gl.compile_shader(f_shader);
        let program = gl.create_program();
        gl.attach_shader(program, v_shader);
        gl.attach_shader(program, f_shader);
        gl.link_program(program);
        gl.use_program(program);
        gl.enable_vertex_attrib_array(0);
        let buffers = gl.gen_buffers(1);
        gl.bind_buffer(gl::ARRAY_BUFFER, buffers[0]);
        gl.buffer_data_untyped(gl::ARRAY_BUFFER, 36, v_vertices.as_ptr() as *const _, gl::STATIC_DRAW);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        Context {
            gl: gl,
            program: program,
            buffer: buffers[0],
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
        emscripten_set_main_loop_arg(loop_wrapper, ptr, 0, 1);
    }
}

const VS_SRC: &'static [&[u8]] = &[b"
attribute mediump vec3 vPosition;
void main() {
    gl_Position = vec4(vPosition, 1.0);
}"];

const FS_SRC: &'static [&[u8]] = &[b"
precision mediump float;
void main() {
    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}"];
