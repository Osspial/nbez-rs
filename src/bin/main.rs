extern crate bev;

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;

use bev::{Bez3o2d};

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
        .with_dimensions(720, 720)
        .with_multisampling(16)
        .with_title("Hello Bézier".into());
    let (window, mut device, mut factory, main_color, _) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let sset = factory.create_shader_set(VERT, FRAG).unwrap();
    let pso = factory.create_pipeline_state(
        &sset,
        Primitive::TriangleStrip,
        Rasterizer::new_fill(),
        pipe::new()
    ).unwrap();

    let curve = Bez3o2d::new(
        0.0, 0.0, 1.0, 1.0,
        0.0, 1.0, 0.0, 1.0
    );

    let mut verts = [Vertex{ pos: [0.0, 0.0] }; 31];
    let mut perps = [Vertex{ pos: [0.0, 0.0] }; 62];
    for i in 0..verts.len() {
        let t = i as f32/(verts.len()-1) as f32;

        let interp = curve.interp(t);
        verts[i].pos = interp.into();

        let perp = curve.slope(t).normalize().perp() * 0.01;
        perps[i*2].pos = (-perp + interp).into();
        perps[i*2 + 1].pos = (perp + interp).into();
    }

    // let (vert_buffer, vert_slice) = factory.create_vertex_buffer_with_slice(&verts, ());
    let (perp_buffer, perp_slice) = factory.create_vertex_buffer_with_slice(&perps, ());
    let mut data = pipe::Data {
        vbuf: perp_buffer.clone(),
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

        // data.vbuf = vert_buffer.clone();
        // encoder.draw(&vert_slice, &pso, &data);

        data.vbuf = perp_buffer.clone();
        encoder.draw(&perp_slice, &pso, &data);

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