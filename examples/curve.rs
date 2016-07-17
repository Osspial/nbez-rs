extern crate nbez;

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;

use nbez::{BezCurve, BezChain, Bez3o, Point2d};

use gfx::traits::FactoryExt;
use gfx::{Factory, Device, Primitive, BufferRole, Bind, Slice, IndexBuffer};
use gfx::state::Rasterizer;

use glutin::{Event, ElementState, MouseButton};

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_vertex_struct!{ Vertex {
    pos: [f32; 2] = "v_pos",
    col: [f32; 3] = "v_col",
}}

gfx_pipeline!{ pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    window_matrix: gfx::Global<[[f32; 2]; 2]> = "window_matrix",
    offset: gfx::Global<[f32; 2]> = "offset",
    out: gfx::RenderTarget<ColorFormat> = "r_target",
}}

fn main() {
    let (mut win_x, mut win_y) = (720, 720);

    let builder = glutin::WindowBuilder::new()
        .with_dimensions(win_x, win_y)
        .with_multisampling(16)
        .with_title("Hello Bezier");
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let sset = factory.create_shader_set(VERT, FRAG).unwrap();
    let pso = factory.create_pipeline_state(
        &sset,
        Primitive::TriangleStrip,
        Rasterizer::new_fill(),
        pipe::new()
    ).unwrap();

    let curve: Bez3o<f32> = Bez3o::new(
        Point2d::new(-0.5, -0.5),
        Point2d::new( 0.5, -0.5),
        Point2d::new(-0.5,  0.5),
        Point2d::new( 0.5,  0.5),
    );

    let (left, right) = curve.split(0.3).unwrap();

    let mut curve_chain: BezChain<f32, Bez3o<f32>, Vec<Point2d<f32>>> = BezChain::from_container(vec![
        left.start,
        left.ctrl0,
        left.ctrl1,
        right.start,
        right.ctrl0,
        right.ctrl1,
        right.end
    ]);

    let radius = 0.02;
    let circle = gen_circle(16, radius, [1.0, 0.0, 0.0]);
    let mut indices = vec![0u16; 42];
    for (ind, i) in indices.iter_mut().enumerate() {
        let ind = ind + 3;
        if ind % 3 == 1 {
            *i = ind as u16/3;
        } else if ind % 3 == 2 {
            *i = ind as u16/3 + 1
        }
    }

    let window_matrix: [[f32; 2]; 2] = [
        [1.0, 0.0],
        [0.0, 1.0],
    ];

    // A vec of curve vertices
    const SAMPLES: usize = 31;
    let mut cverts = [Vertex{ pos: [0.0, 0.0], col: [1.0, 1.0, 1.0] }; SAMPLES * 2];
    let cvert_buffer = factory.create_buffer_dynamic(cverts.len(), BufferRole::Vertex, Bind::empty()).unwrap();
    let cvert_slice = Slice {
        start: 0,
        end: cvert_buffer.len() as u32,
        base_vertex: 0,
        instances: None,
        buffer: IndexBuffer::Auto
    };

    let (cir_buffer, cir_slice) = factory.create_vertex_buffer_with_slice(&circle, &indices[..]);
    let mut data = pipe::Data {
        vbuf: cvert_buffer.clone(),
        window_matrix: window_matrix,
        offset: [0.0, 0.0],
        out: main_color
    };

    // The index of the selected control point. If -1, no control point is selected.
    let mut selected = -1;
    // Mouse x, mouse y
    let (mut mox, mut moy) = (0.0, 0.0);
    'main: loop {
        for event in window.poll_events() {
            match event {
                Event::Closed => break 'main,
                Event::Resized(x, y) => {
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    win_x = x; 
                    win_y = y;
                    data.window_matrix = [
                        [720.0/win_x as f32, 0.0],
                        [0.0, 720.0/win_y as f32]
                    ];
                }
                Event::MouseMoved(x, y) => {
                    mox =  pix_to_float(x, win_x) / data.window_matrix[0][0]; 
                    moy = -pix_to_float(y, win_y) / data.window_matrix[1][1];
                }
                Event::MouseInput(ElementState::Pressed, MouseButton::Left) => {
                    for (i, c) in curve_chain.as_ref().iter().enumerate() {
                        let dist = (c.x-mox).hypot(c.y-moy);
                        if dist <= radius {
                            selected = i as isize;
                        }
                    }
                }
                Event::MouseInput(ElementState::Released, MouseButton::Left) => selected = -1,
                Event::MouseInput(ElementState::Pressed, MouseButton::Right) =>
                    curve_chain.as_mut().push(Point2d::new(mox, moy)),
                _ => ()
            }
        }

        // If any control point is selected, move that control point with the mouse.
        if 0 <= selected {
            let c = &mut curve_chain.as_mut()[selected as usize];
            c.x = mox;
            c.y = moy;
        }


        // Drawing code
        encoder.clear(&data.out, [0.0, 0.0, 0.0, 1.0]);

        for curve in curve_chain.iter() {
            // Calculate curve vertices based on control point position.
            for i in 0..SAMPLES {
                let t = i as f32/(SAMPLES-1) as f32;

                let interp = curve.interp(t).unwrap();

                let perp = curve.slope(t).unwrap().normalize().perp() * 0.01;
                cverts[i*2].pos = (-perp + interp).into();
                cverts[i*2 + 1].pos = (perp + interp).into();
            }
            encoder.update_buffer(&cvert_buffer, &cverts, 0).unwrap(); // Upload calculated vertex positions to vertex buffer


            data.offset = [0.0, 0.0];
            data.vbuf = cvert_buffer.clone();
            encoder.draw(&cvert_slice, &pso, &data);
        }

        // Draw control points
        data.vbuf = cir_buffer.clone();
        for point in curve_chain.as_ref().iter() {
            data.offset = (*point).into();
            encoder.draw(&cir_slice, &pso, &data);
        }


        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

pub fn pix_to_float(p: i32, s: u32) -> f32 {
    let s = s as f32;
    let p = p as f32;
    2.0 * (p - s/2.0)/s
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
    uniform mat2 window_matrix;

    in vec2 v_pos;
    in vec3 v_col;
    out vec4 f_col;

    void main() {
        f_col = vec4(v_col, 1.0);
        gl_Position = vec4(window_matrix * (v_pos + offset), 0.0, 1.0);
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