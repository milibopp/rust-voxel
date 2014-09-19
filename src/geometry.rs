// Cube graphics

use std::slice::Slice;
use std::rand::{XorShiftRng, SeedableRng};
use std::rand::distributions::{Normal, IndependentSample};
use gfx;

use voxel::{Air, Stone, Chunk};


#[vertex_format]
struct Vertex {
    a_pos: [f32, ..3],
    a_color: [f32, ..3],
}

impl Vertex {
    fn new(pos: [f32, ..3], color: [f32, ..3]) -> Vertex {
        Vertex {
            a_pos: pos,
            a_color: color,
        }
    }
}


pub fn make_chunk<C: gfx::CommandBuffer, D: gfx::Device<C> + gfx::DeviceHelper<C>>
    (device: &mut D, pos: (int, int, int), chunk: &Chunk) -> (gfx::Mesh, gfx::Slice)
{
    let simple_cube = [
        (0., 0., 1.),
        (1., 0., 1.),
        (0., 1., 1.),
        (1., 1., 1.),
        (0., 1., 0.),
        (1., 1., 0.),
        (0., 0., 0.),
        (1., 0., 0.),
    ];
    let simple_cube_indices = [
        0u32, 1, 2, 2, 3, 1,
        2, 3, 4, 4, 5, 3,
        0, 1, 6, 6, 7, 1,
        4, 5, 6, 6, 7, 5,
        0, 2, 4, 4, 6, 0,
        1, 3, 5, 5, 7, 1,
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut rng = XorShiftRng::new_unseeded();
    let (i, j, k) = pos;
    rng.reseed([
        (i * j * k) as u32, (k * 166 - i) as u32,
        (j * 99 - i) as u32, 8991
    ]);
    for ((x, y, z), block) in chunk.blocks() {
        match block {
            Stone => {
                let grey = Normal::new(0.5, 0.1).ind_sample(&mut rng) as f32;
                vertices.extend(simple_cube.iter()
                    .map(|&(dx, dy, dz)| Vertex::new(
                        [dx + x as f32, dy + z as f32, dz + y as f32],
                        [grey, grey, grey]
                    ))
                );
                let idx_offset = (vertices.len() - 8) as u32;
                indices.extend(
                    simple_cube_indices.iter()
                    .map(|k| k + idx_offset)
                );
            },
            Air => (),
        }
    }
    (
        device.create_mesh(vertices.as_slice()),
        gfx::IndexSlice32(
            gfx::TriangleList,
            device.create_buffer_static(indices.as_slice()),
            0, indices.len() as u32
        ),
    )
}
