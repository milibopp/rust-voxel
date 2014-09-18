// Cube graphics

use gfx::{
    TriangleList, IndexSlice8, Device, CommandBuffer, DeviceHelper, Slice, Mesh
};


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
