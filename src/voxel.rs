use itertools as it;
use std::iter::Range;
use std::rand::{XorShiftRng, Rng, SeedableRng};
use std::rand::distributions as dist;
use std::rand::distributions::Sample;
use std::collections::HashMap;
use std::num::pow;
use std::iter::AdditiveIterator;


#[deriving(Show)]
pub enum Block {
    Stone,
    Air,
}

pub struct ChunkIterator<'a> {
    chunk: &'a Chunk,
    prod: it::FlatTuples<it::Product<(uint, uint), it::Product<uint, Range<uint>, Range<uint>>, Range<uint>>>,
}

impl<'a> Iterator<((uint, uint, uint), Block)> for ChunkIterator<'a> {
    fn next(&mut self) -> Option<((uint, uint, uint), Block)> {
        match self.prod.next() {
            Some((x, y, z)) => Some(((x, y, z), match self.chunk.data {
                    NormalChunk(data) => data[x][y][z],
                    UniformChunk(block) => block,
                })),
            None => None,
        }
    }
}

enum ChunkData {
    NormalChunk([[[Block, ..16], ..16], ..16]),
    UniformChunk(Block),
}

pub struct Chunk {
    data: ChunkData,
}

impl Chunk {
    pub fn new_uniform(block: Block) -> Chunk {
        Chunk { data: UniformChunk(block) }
    }

    pub fn new_with_data(data: [[[Block, ..16], ..16], ..16]) -> Chunk {
        Chunk { data: NormalChunk(data) }
    }

    pub fn blocks<'a>(&'a self) -> ChunkIterator<'a> {
        ChunkIterator {
            chunk: self,
            prod: iproduct!(range(0u, 16u), range(0u, 16u), range(0u, 16u)),
        }
    }
}


pub struct World {
    landscape: Landscape,
    height_map_cache: HashMap<(int, int), [[int, ..16], ..16]>,
    chunk_cache: HashMap<(int, int, int), Chunk>,
}

impl World {
    pub fn new(landscape: Landscape) -> World {
        World {
            landscape: landscape,
            height_map_cache: HashMap::new(),
            chunk_cache: HashMap::new(),
        }
    }

    pub fn get_chunk<'a>(&'a mut self, coord: (int, int, int)) -> &'a Chunk {
        let (i, j, k) = coord;
        // Has that chunk already been generated?
        let hm_cache = &mut self.height_map_cache;
        let landscape = &mut self.landscape;
        self.chunk_cache.find_or_insert_with(coord, |&coord| {
            // Let's see if we have at least a height map cached…
            let height_map = hm_cache.find_or_insert_with(
                (i, j), |&(i, j)|
            {
                let mut height_map = [[0i, ..16], ..16];
                // Bilinear interpolation
                let corners: Vec<((int, int), f64)> =
                    iproduct!(range(0i, 2), range(0i, 2))
                    .map(|(ii, jj)| (
                        (ii * 16, jj * 16),
                        landscape
                        .get(((i + ii) as uint, (j + jj) as uint))
                        .unwrap()
                    ))
                    .collect();
                for (x, y) in iproduct!(range(0i, 16), range(0i, 16)) {
                    height_map[x as uint][y as uint] =
                        (corners.iter().map(|&((xi, yi), q)|
                            q * (x - xi).abs() as f64
                              * (y - yi).abs() as f64
                        )
                        .sum() / 256.0) as int;
                }
                // Perlin noise
                /*for scale in range(1u, 4).rev().map(|k| pow(2u, k)) {
                    let n = 16 / scale;
                    for (cx, cy) in iproduct!(range(0, n), range(0, n)) {
                        let rng: XorShiftRng = SeedableRng::from_seed([
                            scale as u32, cx as u32, cy as u32,
                            (i + j) as u32
                        ]);
                        let (xs, ys) = (i * 16, j * 16);
                        for (x, y) in iproduct!(
                            range(xs, xs + scale as int),
                            range(ys, ys + scale as int)
                        ) {
                            height_data[][] += blah foo
                        }
                    }
                }*/
                height_map
            });
            // Now generate the chunk from it
            let max = *height_map.iter().map(|row| row.iter().max().unwrap()).max().unwrap();
            let min = *height_map.iter().map(|row| row.iter().min().unwrap()).min().unwrap();
            if k * 16 > max {
                Chunk::new_uniform(Air)
            } else if (k + 1) * 16 <= min {
                Chunk::new_uniform(Stone)
            } else {
                let mut data = [[[Air, ..16], ..16], ..16];
                for (x, y) in iproduct!(range(0i, 16), range(0i, 16)) {
                    let h = height_map[(i * 16 + x) as uint][(j * 16 + y) as uint] - k * 16;
                    for z in range(0, h) {
                        data[x as uint][y as uint][z as uint] = Stone;
                    }
                }
                Chunk::new_with_data(data)
            }
        })
    }
}


pub struct Landscape {
    height_data: Vec<f64>,
    dims: (uint, uint),
}

impl Landscape {
    pub fn generate<S, R: Rng + SeedableRng<S>>(rng: &mut R, dims: (uint, uint)) -> Landscape {
        let (x, y) = dims;
        let mut height_range = dist::Range::new(0.0f64, 3.0);
        Landscape {
            height_data: range(0, x * y)
                .map(|_| height_range.sample(rng))
                .collect(),
            dims: dims,
        }
    }

    pub fn get(&self, pos: (uint, uint)) -> Option<f64> {
        let (x, y) = pos;
        let (dx, dy) = self.dims;
        if x < dx && y < dy {
            Some(self.height_data[x * dx + y])
        }
        else {
            None
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_world_get_chunk() {
    
    }

}
