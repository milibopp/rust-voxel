#![feature(phase)]
#![feature(globs)]
#![crate_name = "cube"]

extern crate gfx;
extern crate piston;
// extern crate glfw_game_window;
extern crate sdl2_game_window;
#[phase(plugin)]
extern crate gfx_macros;
extern crate native;
extern crate time;
#[phase(plugin, link)]
extern crate itertools;

// use glfw_game_window::WindowGLFW;
use sdl2_game_window::WindowSDL2;
use gfx::{Device, DeviceHelper};
use piston::{cam, Window};
use piston::input::{Keyboard, keyboard};
use std::num::One;
use itertools as it;
use std::iter::Range;
//----------------------------------------
// Cube associated data

#[vertex_format]
struct Vertex {
    #[as_float]
    a_pos: [f32, ..3],
    #[as_float]
    a_tex_coord: [f32, ..2],
}

impl Vertex {
    fn new(pos: [f32, ..3], tc: [f32, ..2]) -> Vertex {
        Vertex {
            a_pos: pos,
            a_tex_coord: tc,
        }
    }
}

#[shader_param(CubeBatch)]
struct Params {
    u_model_view_proj: [[f32, ..4], ..4],
    t_color: gfx::shade::TextureParam,
}

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_120: b"
    #version 120
    attribute vec3 a_pos;
    attribute vec2 a_tex_coord;
    varying vec2 v_TexCoord;
    uniform mat4 u_model_view_proj;
    void main() {
        v_TexCoord = a_tex_coord;
        gl_Position = u_model_view_proj * vec4(a_pos, 1.0);
    }
"
GLSL_150: b"
    #version 150 core
    in vec3 a_pos;
    in vec2 a_tex_coord;
    out vec2 v_TexCoord;
    uniform mat4 u_model_view_proj;
    void main() {
        v_TexCoord = a_tex_coord;
        gl_Position = u_model_view_proj * vec4(a_pos, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_120: b"
    #version 120
    varying vec2 v_TexCoord;
    uniform sampler2D t_color;
    void main() {
        vec4 tex = texture2D(t_color, v_TexCoord);
        float blend = dot(v_TexCoord-vec2(0.5,0.5), v_TexCoord-vec2(0.5,0.5));
        gl_FragColor = mix(tex, vec4(0.0,0.0,0.0,0.0), blend*1.0);
    }
"
GLSL_150: b"
    #version 150 core
    in vec2 v_TexCoord;
    out vec4 o_Color;
    uniform sampler2D t_color;
    void main() {
        vec4 tex = texture(t_color, v_TexCoord);
        float blend = dot(v_TexCoord-vec2(0.5,0.5), v_TexCoord-vec2(0.5,0.5));
        o_Color = mix(tex, vec4(0.0,0.0,0.0,0.0), blend*1.0);
    }
"
};

//----------------------------------------

// We need to run on the main thread, so ensure we are using the `native` runtime. This is
// technically not needed, since this is the default, but it's not guaranteed.
#[start]
fn start(argc: int, argv: *const *const u8) -> int {
     native::start(argc, argv, main)
}


enum Block {
    Stone,
    Air,
}

struct ChunkIterator<'a> {
    chunk: &'a Chunk,
    prod: it::FlatTuples<it::Product<(uint, uint), it::Product<uint, Range<uint>, Range<uint>>, Range<uint>>>,
}

impl<'a> Iterator<((uint, uint, uint), Block)> for ChunkIterator<'a> {
    fn next(&mut self) -> Option<((uint, uint, uint), Block)> {
        match self.prod.next() {
            Some((x, y, z)) => Some(((x, y, z), self.chunk.data[x][y][z])),
            None => None,
        }
    }

}

struct Chunk {
    data: [[[Block, ..16], ..16], ..16],
}

impl Chunk {
    fn new() -> Chunk {
        let mut data = [[[Air, ..16], ..16], ..16];
        for (x, y, z) in iproduct!(range(0u, 16u), range(0u, 16u), range(10u, 16u)) {
            data[x][y][z] = Stone;
        }
        Chunk { data: data }
    }

    fn blocks<'a>(&'a self) -> ChunkIterator<'a> {
        ChunkIterator {
            chunk: self,
            prod: iproduct!(range(0u, 16u), range(0u, 16u), range(0u, 16u)),
        }
    }
}


/*struct World {
    chunks: Vec<((i64, i64), Chunk)>,
}*/


fn main() {
    let (win_width, win_height) = (1920, 1080);
    let mut window = WindowSDL2::new(
        piston::shader_version::opengl::OpenGL_3_2,
        piston::WindowSettings {
            title: "cube".to_string(),
            size: [win_width, win_height],
            fullscreen: true,
            exit_on_esc: true,
            samples: 4,
        }
    );

    window.capture_cursor(true);

    let (mut device, frame) = window.gfx();
    let state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);

    let basic_cube = vec![
        //top (0., 0., 1.)
        Vertex::new([-1., -1.,  1.], [0., 0.]),
        Vertex::new([ 1., -1.,  1.], [1., 0.]),
        Vertex::new([ 1.,  1.,  1.], [1., 1.]),
        Vertex::new([-1.,  1.,  1.], [0., 1.]),
        //bottom (0., 0., -1.)
        Vertex::new([ 1.,  1., -1.], [0., 0.]),
        Vertex::new([-1.,  1., -1.], [1., 0.]),
        Vertex::new([-1., -1., -1.], [1., 1.]),
        Vertex::new([ 1., -1., -1.], [0., 1.]),
        //right (1., 0., 0.)
        Vertex::new([ 1., -1., -1.], [0., 0.]),
        Vertex::new([ 1.,  1., -1.], [1., 0.]),
        Vertex::new([ 1.,  1.,  1.], [1., 1.]),
        Vertex::new([ 1., -1.,  1.], [0., 1.]),
        //left (-1., 0., 0.)
        Vertex::new([-1.,  1.,  1.], [0., 0.]),
        Vertex::new([-1., -1.,  1.], [1., 0.]),
        Vertex::new([-1., -1., -1.], [1., 1.]),
        Vertex::new([-1.,  1., -1.], [0., 1.]),
        //front (0., 1., 0.)
        Vertex::new([-1.,  1., -1.], [0., 0.]),
        Vertex::new([ 1.,  1., -1.], [1., 0.]),
        Vertex::new([ 1.,  1.,  1.], [1., 1.]),
        Vertex::new([-1.,  1.,  1.], [0., 1.]),
        //back (0., -1., 0.)
        Vertex::new([ 1., -1.,  1.], [0., 0.]),
        Vertex::new([-1., -1.,  1.], [1., 0.]),
        Vertex::new([-1., -1., -1.], [1., 1.]),
        Vertex::new([ 1., -1., -1.], [0., 1.]),
    ];
    let vertex_data = {
        let mut data = Vec::new();
        for i in range(0u, 10) {
            data.extend(
                basic_cube.iter()
                    .map(|v| {
                        Vertex::new(
                            [
                                v.a_pos[0] * 0.5 + (i as f32),
                                v.a_pos[1] * 0.5,
                                v.a_pos[2] * 0.5,
                            ],
                            v.a_tex_coord
                        )
                    })
            );
        }
        data
    };

    let mesh = device.create_mesh(vertex_data);

    let slice = {
        let mut index_data = Vec::new();
        for i in range(0u32, 10) {
            index_data.extend(
                vec![
                    0, 1, 2, 2, 3, 0,    //top
                    4, 5, 6, 6, 7, 4,       //bottom
                    8, 9, 10, 10, 11, 8,    //right
                    12, 13, 14, 14, 16, 12, //left
                    16, 17, 18, 18, 19, 16, //front
                    20, 21, 22, 22, 23, 20, //back
                ]
                .iter()
                .map(|&index| index + (i * 24))
            );
        }

        let buf = device.create_buffer_static(&index_data);
        gfx::IndexSlice32(gfx::TriangleList, buf, 0, index_data.len() as u32)
    };

    let tinfo = gfx::tex::TextureInfo {
        width: 1,
        height: 1,
        depth: 1,
        levels: 1,
        kind: gfx::tex::Texture2D,
        format: gfx::tex::RGBA8,
    };
    let img_info = tinfo.to_image_info();
    let texture = device.create_texture(tinfo).unwrap();
    device.update_texture(
            &texture, 
            &img_info,
            &vec![0x20u8, 0xA0u8, 0xC0u8, 0x00u8].as_slice()
        ).unwrap();

    let sampler = device.create_sampler(
        gfx::tex::SamplerInfo::new(
            gfx::tex::Bilinear, 
            gfx::tex::Clamp
        )
    );
    
    let program = device.link_program(
            VERTEX_SRC.clone(), 
            FRAGMENT_SRC.clone()
        ).unwrap();

    let mut graphics = gfx::Graphics::new(device);
    let batch: CubeBatch = graphics.make_batch(&program, &mesh, slice, &state).unwrap();

    let mut data = Params {
        u_model_view_proj: piston::vecmath::mat4_id(),
        t_color: (texture, Some(sampler)),
    };

    let model = piston::vecmath::mat4_id();
    let projection = cam::CameraPerspective {
            fov: 70.0f32,
            near_clip: 0.1,
            far_clip: 1000.0,
            aspect_ratio: (win_width as f32) / (win_height as f32)
        }.projection();
    let mut first_person = cam::FirstPerson::new(
        [0.5f32, 0.5, 4.0],
        cam::FirstPersonSettings{
            move_forward_button: Keyboard(keyboard::V),
            move_backward_button: Keyboard(keyboard::I),
            strafe_left_button: Keyboard(keyboard::U),
            strafe_right_button: Keyboard(keyboard::A),
            fly_up_button: Keyboard(keyboard::Space),
            fly_down_button: Keyboard(keyboard::LShift),
            move_faster_button: Keyboard(keyboard::LCtrl),
            speed_horizontal: One::one(),
            speed_vertical: One::one(),
        }
    );

    let mut game_iter = piston::EventIterator::new(
        &mut window,
        &piston::EventSettings {
            updates_per_second: 120,
            max_frames_per_second: 60
        }
    );

    for e in game_iter {
        match e {
            piston::Render(args) => {
                graphics.clear(
                    gfx::ClearData {
                        color: [0.0, 0.0, 0.0, 1.0],
                        depth: 1.0,
                        stencil: 0,
                    },
                    gfx::Color | gfx::Depth,
                    &frame
                );
                data.u_model_view_proj = cam::model_view_projection(
                        model,
                        first_person.camera(args.ext_dt).orthogonal(),
                        projection
                    );
                graphics.draw(&batch, &data, &frame);
                graphics.end_frame();
            },
            piston::Update(args) => first_person.update(args.dt),
            piston::Input(e) => first_person.input(&e),
        }
    }
}


