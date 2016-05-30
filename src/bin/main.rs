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
    col: [f32; 3] = "v_col",
}}

gfx_pipeline!{ pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    offset: gfx::Global<[f32; 2]> = "offset",
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

    let mut curve = Bez3o2d::new(
        -0.5, -0.5,
        -0.5,  0.5,
         0.5, -0.5,
         0.5,  0.5
    );

    const SAMPLES: usize = 31;
    let mut cverts = [Vertex{ pos: [0.0, 0.0], col: [1.0, 1.0, 1.0] }; SAMPLES * 2];
    for i in 0..SAMPLES {
        let t = i as f32/(SAMPLES-1) as f32;

        let interp = curve.interp(t);

        let perp = curve.slope(t).normalize().perp() * 0.01;
        cverts[i*2].pos = (-perp + interp).into();
        cverts[i*2 + 1].pos = (perp + interp).into();
    }

    let circle = gen_circle(16, 0.02, [1.0, 0.0, 0.0]);
    let mut indices = vec![0u16; 42];
    for (ind, i) in indices.iter_mut().enumerate() {
        let ind = ind + 3;
        if ind % 3 == 1 {
            *i = ind as u16/3;
        } else if ind % 3 == 2 {
            *i = ind as u16/3 + 1
        }
    }

    let (cvert_buffer, cvert_slice) = factory.create_vertex_buffer_with_slice(&cverts, ());
    let (cir_buffer, cir_slice) = factory.create_vertex_buffer_with_slice(&circle, &indices[..]);
    let mut data = pipe::Data {
        vbuf: cvert_buffer.clone(),
        offset: [0.0, 0.0],
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

        data.offset = [0.0, 0.0];
        data.vbuf = cvert_buffer.clone();
        encoder.draw(&cvert_slice, &pso, &data);

        data.vbuf = cir_buffer.clone();
        data.offset = curve.start.into();
        encoder.draw(&cir_slice, &pso, &data);
        data.offset = curve.ctrl0.into();
        encoder.draw(&cir_slice, &pso, &data);
        data.offset = curve.ctrl1.into();
        encoder.draw(&cir_slice, &pso, &data);
        data.offset = curve.end.into();
        encoder.draw(&cir_slice, &pso, &data);


        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

fn gen_circle(divs: u16, scale: f32, col: [f32; 3]) -> Vec<Vertex> {
    use std::f32::consts::PI;
    let mut verts = vec![Vertex{ pos: [0.0, 0.0], col: col}; divs as usize];

    for d in 0..divs {
        let theta = 2.0 * PI * (d as f32/divs as f32);
        let d = d as usize;
        verts[d].pos[0] = theta.sin() * scale;
        verts[d].pos[1] = theta.cos() * scale;
    }

    verts
}

const VERT: &'static [u8] = br#"
    #version 150 core

    uniform vec2 offset;

    in vec2 v_pos;
    in vec3 v_col;
    out vec4 f_col;

    void main() {
        f_col = vec4(v_col, 1.0);
        gl_Position = vec4(v_pos + offset, 0.0, 1.0);
    }
"#;

const FRAG: &'static [u8] = br#"
    #version 150 core

    in vec4 f_col;
    out vec4 r_target;

    void main() {
        r_target = f_col;
    }
"#;