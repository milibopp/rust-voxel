#![feature(phase)]
#![feature(globs)]
#![crate_name = "cube"]

extern crate gfx;
extern crate piston;
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
use std::num::{One, Float};
use voxel::{Air, Stone, Landscape};
use std::rand::{Rng, SeedableRng, XorShiftRng};

pub mod voxel;

// Cube associated data
#[vertex_format]
struct Vertex {
    #[as_float]
    a_pos: [f32, ..3],
}

impl Vertex {
    fn new(pos: [f32, ..3]) -> Vertex {
        Vertex {
            a_pos: pos,
        }
    }
}

#[shader_param(CubeBatch)]
struct Params {
    u_model_view_proj: [[f32, ..4], ..4],
    t_color: [f32, ..3],
}

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    in vec3 a_pos;
    uniform mat4 u_model_view_proj;
    void main() {
        gl_Position = u_model_view_proj * vec4(a_pos, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    out vec4 o_Color;
    uniform vec3 t_color;
    void main() {
        o_Color = vec4(t_color, 1.0);
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


fn main() {
    // Basic window setup
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

    // Generate geometry (this requires a device)
    let basic_cube = vec![
        Vertex::new([0., 0., 1.]),
        Vertex::new([1., 0., 1.]),
        Vertex::new([0., 1., 1.]),
        Vertex::new([1., 1., 1.]),
        Vertex::new([0., 1., 0.]),
        Vertex::new([1., 1., 0.]),
        Vertex::new([0., 0., 0.]),
        Vertex::new([1., 0., 0.]),
    ];

    let mesh = device.create_mesh(basic_cube);

    let slice = {
        let index_data = vec![
            0u8, 1, 2, 2, 3, 1,
            2, 3, 4, 4, 5, 3,
            0, 1, 6, 6, 7, 1,
            4, 5, 6, 6, 7, 5,
            0, 2, 4, 4, 6, 0,
            1, 3, 5, 5, 7, 1,
        ];
        let buf = device.create_buffer_static(&index_data);
        gfx::IndexSlice8(gfx::TriangleList, buf, 0, index_data.len() as u32)
    };

    let program = device.link_program(
            VERTEX_SRC.clone(), 
            FRAGMENT_SRC.clone()
        ).unwrap();

    let mut graphics = gfx::Graphics::new(device);
    let batch: CubeBatch = graphics.make_batch(&program, &mesh, slice, &state).unwrap();

    // Game state data
    let mut rng: XorShiftRng = SeedableRng::from_seed([2, 3, 5, 8]);
    let scape = Landscape::generate(&mut rng, (16, 16));

    // Camera handling
    let projection = cam::CameraPerspective {
            fov: 70.0f32,
            near_clip: 0.1,
            far_clip: 1000.0,
            aspect_ratio: (win_width as f32) / (win_height as f32)
        }.projection();
    let mut first_person = cam::FirstPerson::new(
        [8.0f32, 6.0, 8.0],
        cam::FirstPersonSettings{
            move_forward_button: Keyboard(keyboard::V),
            move_backward_button: Keyboard(keyboard::I),
            strafe_left_button: Keyboard(keyboard::U),
            strafe_right_button: Keyboard(keyboard::A),
            fly_up_button: Keyboard(keyboard::Space),
            fly_down_button: Keyboard(keyboard::LShift),
            move_faster_button: Keyboard(keyboard::LCtrl),
            speed_horizontal: 10.0,
            speed_vertical: 10.0,
        }
    );

    // Iteration loop
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
                for x in range(0u, 16) { for z in range(0u, 16) {
                    let y = scape.height_data[x * 16 + z] as f32;
                    let model = [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [x as f32, y.round(), z as f32, 1.0],
                    ];
                    let light = (y / 3. + 0.1);
                    let data = Params{
                        u_model_view_proj: cam::model_view_projection(
                            model,
                            first_person.camera(args.ext_dt).orthogonal(),
                            projection
                        ),
                        t_color: [0.5 * light, (0.47 - y * 0.015) * light, 0.40 * light],
                    };
                    graphics.draw(&batch, &data, &frame);
                }}
                graphics.end_frame();
            },
            piston::Update(args) => {
                first_person.update(args.dt);
            },
            piston::Input(e) => first_person.input(&e),
        }
    }
}
