extern crate bev;

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;


use bev::{BezCube};
use bev::core::BezCubePoly;

use gfx::traits::FactoryExt;
use gfx::{Device, Primitive};
use gfx::state::Rasterizer;

use glutin::Event;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_vertex_struct!{ Vertex {
    pos: [f32; 2] = "v_pos",
}}

gfx_pipeline!{ pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    out: gfx::RenderTarget<ColorFormat> = "r_target",
}}

fn main() {
    let builder = glutin::WindowBuilder::new()
        .with_title("Hello Square".into());
    let (window, mut device, mut factory, main_color, _) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let sset = factory.create_shader_set(VERT, FRAG).unwrap();
    let pso = factory.create_pipeline_state(
        &sset,
        Primitive::LineStrip,
        Rasterizer::new_fill(),
        pipe::new()
    ).unwrap();


    let curve = BezCube {
        x: BezCubePoly::new(0.0, 0.0, 1.0, 1.0),
        y: BezCubePoly::new(0.0, 1.0, 0.0, 1.0)
    };

    let mut verts = [Vertex{ pos: [0.0, 0.0] }; 31];
    for i in 0..verts.len() {
        verts[i].pos = curve.interp(i as f32/(verts.len()-1) as f32).into();
    }

    let (vert_buffer, slice) = factory.create_vertex_buffer_with_slice(&verts, ());
    let data = pipe::Data {
        vbuf: vert_buffer,
        out: main_color
    };

    'main: loop {
        for event in window.poll_events() {
            match event {
                Event::Closed => break 'main,
                _ => ()
            }
        }

        encoder.clear(&data.out, [0.0, 0.0, 0.0, 1.0]);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

const VERT: &'static [u8] = br#"
    #version 150 core

    in vec2 v_pos;

    void main() {
        gl_Position = vec4(v_pos, 0.0, 1.0);
    }
"#;

const FRAG: &'static [u8] = br#"
    #version 150 core

    out vec4 r_target;

    void main() {
        r_target = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;