#![feature(phase)]
#![feature(globs)]
#![crate_name = "cube"]

extern crate gfx;
extern crate piston;
extern crate sdl2;
extern crate sdl2_game_window;
#[phase(plugin)]
extern crate gfx_macros;
extern crate native;
extern crate time;
#[phase(plugin, link)]
extern crate itertools;


// use glfw_game_window::WindowGLFW;
use sdl2_game_window::WindowSDL2;
use gfx::{Mesh, Device, DeviceHelper};
use piston::{cam, Window};
use piston::input::{Keyboard, keyboard};
use std::num::Float;
use std::rand::{SeedableRng, XorShiftRng};
use std::collections::HashMap;

use voxel::{Stone, Air, World, Landscape};
use geometry::make_chunk;

pub mod voxel;
pub mod geometry;

// Cube associated data
#[shader_param(CubeBatch)]
struct Params {
    u_model_view_proj: [[f32, ..4], ..4],
}

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    in vec3 a_pos;
    in vec3 a_color;
    out vec3 v_color;
    uniform mat4 u_model_view_proj;
    void main() {
        gl_Position = u_model_view_proj * vec4(a_pos, 1.0);
        v_color = a_color;
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    in vec3 v_color;
    out vec4 o_Color;
    void main() {
        o_Color = vec4(v_color, 1.0);
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

    let mut device = gfx::GlDevice::new(|s| unsafe {
        std::mem::transmute(sdl2::video::gl_get_proc_address(s))
    });
    let frame = gfx::Frame::new(win_width as u16, win_height as u16);
    let state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);

    // Generate geometry (this requires a device)
    let program = device.link_program(
            VERTEX_SRC.clone(), 
            FRAGMENT_SRC.clone()
        ).unwrap();
    let mut graphics = gfx::Graphics::new(device);

    // Game state data
    let mut rng: XorShiftRng = SeedableRng::from_seed([2, 3, 5, 8]);
    let scape = Landscape::generate(&mut rng, (16, 16));
    let mut world = World::new(scape);

    // Cache for rendered chunks
    let mut render_chunk_cache = HashMap::new();

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
                let p: Vec<int> = first_person.position.iter().map(|x| (x / 16.).round() as int).collect();
                let render_coords: Vec<(int, int, int)> = iproduct!(
                        range(p[0] - 1, p[0] + 2),
                        range(p[1] - 1, p[1] + 2),
                        range(p[2] - 4, p[2] + 2))
                    .collect();
                for &coord in render_coords.iter() {
                    render_chunk_cache.find_or_insert_with(coord, |&c| {
                        let chunk = world.get_chunk(c);
                        let (mesh, slice) = make_chunk(&mut graphics.device, c, chunk);
                        let batch: CubeBatch = graphics.make_batch(
                            &program, &mesh, slice, &state).unwrap();
                        batch
                    });
                }
                for &(i, j, k) in render_coords.iter() {
                    let batch = render_chunk_cache.find(&(i, j, k)).unwrap();
                    let model = [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [
                            (i * 16) as f32,
                            (k * 16) as f32,
                            (j * 16) as f32,
                            1.0
                        ],
                    ];
                    let data = Params{
                        u_model_view_proj: cam::model_view_projection(
                            model,
                            first_person.camera(args.ext_dt).orthogonal(),
                            projection
                        ),
                    };
                    graphics.draw(batch, &data, &frame);
                }
                graphics.end_frame();
            },
            piston::Update(args) => {
                first_person.update(args.dt);
            },
            piston::Input(e) => first_person.input(&e),
        }
    }
}
