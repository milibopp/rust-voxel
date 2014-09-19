// Cube graphics

use gfx::{
    TriangleList, IndexSlice8, IndexSlice16, Device, CommandBuffer,
    DeviceHelper, Slice, Mesh
};

use voxel::{Air, Stone, Chunk};


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


pub fn make_cube<C: CommandBuffer, D: Device<C> + DeviceHelper<C>>
    (device: &mut D) -> (Mesh, Slice)
{
    let vertices = vec![
        Vertex::new([0., 0., 1.]),
        Vertex::new([1., 0., 1.]),
        Vertex::new([0., 1., 1.]),
        Vertex::new([1., 1., 1.]),
        Vertex::new([0., 1., 0.]),
        Vertex::new([1., 1., 0.]),
        Vertex::new([0., 0., 0.]),
        Vertex::new([1., 0., 0.]),
    ];
    let indices = vec![
        0u8, 1, 2, 2, 3, 1,
        2, 3, 4, 4, 5, 3,
        0, 1, 6, 6, 7, 1,
        4, 5, 6, 6, 7, 5,
        0, 2, 4, 4, 6, 0,
        1, 3, 5, 5, 7, 1,
    ];
    (
        device.create_mesh(vertices),
        IndexSlice8(
            TriangleList,
            device.create_buffer_static(&indices),
            0, indices.len() as u32
        ),
    )
}


pub fn make_chunk<C: CommandBuffer, D: Device<C> + DeviceHelper<C>>
    (device: &mut D, chunk: &Chunk) -> (Mesh, Slice)
{
    let simple_cube = vec![
        Vertex::new([0., 0., 1.]),
        Vertex::new([1., 0., 1.]),
        Vertex::new([0., 1., 1.]),
        Vertex::new([1., 1., 1.]),
        Vertex::new([0., 1., 0.]),
        Vertex::new([1., 1., 0.]),
        Vertex::new([0., 0., 0.]),
        Vertex::new([1., 0., 0.]),
    ];
    let simple_cube_indices = vec![
        0u16, 1, 2, 2, 3, 1,
        2, 3, 4, 4, 5, 3,
        0, 1, 6, 6, 7, 1,
        4, 5, 6, 6, 7, 5,
        0, 2, 4, 4, 6, 0,
        1, 3, 5, 5, 7, 1,
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for ((x, y, z), block) in chunk.blocks() {
        match block {
            Stone => {
                vertices.extend(simple_cube.iter()
                    .map(|v| Vertex::new([
                        v.a_pos[0] + x as f32,
                        v.a_pos[1] + z as f32,
                        v.a_pos[2] + y as f32,
                    ]))
                );
                let idx_offset = (vertices.len() - 8) as u16;
                indices.extend(
                    simple_cube_indices.iter()
                    .map(|k| k + idx_offset)
                );
            },
            Air => (),
        }
    }

    (
        device.create_mesh(vertices),
        IndexSlice16(
            TriangleList,
            device.create_buffer_static(&indices),
            0, indices.len() as u32
        ),
    )
}
