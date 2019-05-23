use super::Lattice;

use graphics::math;
use model::SandPileModel;
use model::region::Cuboid;
use model::sand_graph::{SandGraph, NodeIndex};
use model::embedding::{ EmbeddingToR3, Figure };
use super::tilings::{ tiling_square, tiling_hexagon, tiling_triangle };


pub struct SquareLattice {}
pub struct HexagonLattice {}
pub struct TriangleLattice {}
pub struct CubeLattice {}

impl SquareLattice {
    pub fn new() -> SquareLattice {
        SquareLattice {}
    }
}

impl Lattice for SquareLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        tiling_square(cuboid_hull, 0)
    }
}

impl HexagonLattice {
    pub fn new() -> HexagonLattice {
        HexagonLattice {}
    }
}

impl Lattice for HexagonLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        tiling_hexagon(cuboid_hull, 0)
    }
}

impl TriangleLattice {
    pub fn new() -> TriangleLattice {
        TriangleLattice {}
    }
}

impl Lattice for TriangleLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        tiling_triangle(cuboid_hull, 0)
    }
}

impl CubeLattice {
    pub fn new() -> Self {
        CubeLattice {}
    }

    fn cube() -> Figure {
        let s = 0.5_f32;  // half of side length
        let vertices = vec![
            [-s, -s, s], [-s, s, s], [s, s, s], [s, -s, s],         // up square
            [-s, -s, -s], [-s, s, -s], [s, s, -s], [s, -s, -s],     // down square
        ];
        let indexes = vec![
            0, 1, 2,  0, 2, 3,  // up
            4, 5, 6,  4, 6, 7,  // down
            0, 1, 5,  0, 5, 4,  // left
            0, 4, 7,  0, 7, 3,  // front
            3, 7, 6,  3, 6, 2,  // right
            1, 2, 6,  1, 6, 5,  // far
        ];
        let border_indexes = vec![
            0, 1,  2, 3,  1, 2,  0, 3,  // up
            4, 5,  6, 7,  5, 6,  4, 7,  // down
            0, 4,  1, 5,  2, 6,  3, 7,  // side
        ];
        Figure { vertices, indexes, border_indexes }
    }
}


impl Lattice for CubeLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        let x_size = cuboid_hull[0] as usize;
        let y_size = cuboid_hull[1] as usize;
        let z_size = cuboid_hull[2] as usize;
        let edges_count = x_size*y_size*z_size;

        let coords_to_index =
            |x: usize, y: usize, z: usize| { z*x_size*y_size + y*x_size + x + 1};

        let mut sand_graph = SandGraph::new();
        let mut embedding = EmbeddingToR3::new();

        let figure = Self::cube();
        embedding.add_figure(figure);


        for _ in 0..edges_count {
            sand_graph.add_node();
        }
        let mut neighbours= [
            (1, 0, 0),
            (0, 1, 0),
            (-1, 0, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];
        for x in 0..x_size {
            for y in 0..y_size {
                for z in 0..z_size {
                    let this_node_idx = coords_to_index(x, y, z);
                    for (dx, dy, dz) in neighbours.iter() {
                        let (nx, ny, nz) = (x as i32 + *dx, y as i32 + *dy, z as i32 + *dz);
                        let neighbour_node_idx = coords_to_index(nx as usize, ny as usize, nz as usize);
                        if 0 <= nx && nx < x_size as i32 &&
                            0 <= ny && ny < y_size as i32 &&
                            0 <= nz && nz < z_size as i32 {
                            sand_graph.add_edge(this_node_idx, neighbour_node_idx, 1);
                        } else {
                            sand_graph.add_edge(this_node_idx, SandGraph::SINK_NODE, 1);
                        }
                    }

                    embedding.set_node_info(this_node_idx, [x as f32, y as f32, z as f32], 0);
                }
            }
        }

        SandPileModel {graph: sand_graph, embedding }
    }
}
