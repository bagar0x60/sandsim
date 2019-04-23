use graphics::math;
use model::SandPileModel;
use super::region::Cuboid;
use super::sand_graph::{SandGraph, NodeIndex};
use super::embedding::EmbeddingToR3;

pub trait Lattice2D {
    fn get_lattice_2d(&self, cuboid_hull: &Cuboid) -> SandPileModel;
}

pub trait Lattice3D {
    fn get_lattice_3d(cuboid_hull: &Cuboid) -> SandPileModel;
}

pub struct SquareLattice {}
pub struct HexagonLattice {}
pub struct TriangleLattice {}

impl SquareLattice {
    pub fn new() -> SquareLattice {
        SquareLattice {}
    }
}

impl Lattice2D for SquareLattice{
    fn get_lattice_2d(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        let x_size = (cuboid_hull[0] as usize);
        let y_size = (cuboid_hull[1] as usize);
        let edges_count = x_size*y_size;

        let coords_to_index = |x: usize, y: usize| -> NodeIndex { y*x_size + x + 1};

        let mut sand_graph = SandGraph::new();

        for _ in 0..edges_count {
            sand_graph.add_node();
        }

        let mut neighbours: [NodeIndex; 4];
        let mut embedding: Vec<math::Vec3d> = vec![[0.0, 0.0, 0.0]; edges_count + 1];

        for x in 0..x_size {
            for y in 0..y_size {
                neighbours = [SandGraph::SINK_NODE; 4];
                if x > 0 { neighbours[0] = coords_to_index(x - 1, y) };
                if x < x_size - 1 { neighbours[1] = coords_to_index(x + 1, y) };
                if y > 0 { neighbours[2] = coords_to_index(x, y - 1) };
                if y < y_size - 1 { neighbours[3] = coords_to_index(x, y + 1) };

                let this_node = coords_to_index(x, y);
                for neighbour in &neighbours {
                    sand_graph.add_edge(this_node, *neighbour, 1);
                }

                embedding[this_node] = [x as f64, y as f64, 0.0];
            }
        }

        SandPileModel {graph: sand_graph, embedding: EmbeddingToR3::new(embedding) }
    }
}


impl HexagonLattice {
    pub fn new() -> HexagonLattice {
        HexagonLattice {}
    }
}

impl Lattice2D for HexagonLattice {
    fn get_lattice_2d(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        let x_size = cuboid_hull[0];
        let y_size = cuboid_hull[1];

        let mut sand_graph = SandGraph::new();
        let mut embedding: Vec<math::Vec3d> = vec![[0.0, 0.0, 0.0]];

        let mut nodes_arrangement_scheme: Vec<Vec<NodeIndex>> = Vec::new();

        let mut iy: usize = 0;
        loop {
            let y = (0.75_f64).powf(0.5) * (iy as f64);
            if y > y_size {
                break
            }

            nodes_arrangement_scheme.push(Vec::new());

            let mut ix: usize = 0;
            loop {
                let x = (ix as f64) + 0.5*((iy % 2) as f64);
                if x > x_size {
                    break
                }

                let node_idx = sand_graph.add_node();
                embedding.push([x, y, 0.0]);
                nodes_arrangement_scheme[iy].push(node_idx);
                ix += 1;
            }
            iy += 1;
        }

        let neighbours = [(1, 0), (-1, 0), (-1, -1), (0, -1), (-1, 1), (0, 1)];

        for iy in 0..nodes_arrangement_scheme.len() {
            for ix in 0..nodes_arrangement_scheme[iy].len() {
                let current_code_idx = nodes_arrangement_scheme[iy][ix];

                let sign = match iy % 2 {
                    0 => 1,
                    _ => -1
                };

                for (dx, dy) in neighbours.iter() {
                    let nx = (ix as i32) + sign*(*dx);
                    let ny = (iy as i32) + sign*(*dy);

                    if 0 <= ny && ny < nodes_arrangement_scheme.len() as i32 &&
                        0 <= nx && nx < nodes_arrangement_scheme[ny as usize].len() as i32 {
                        let neighbour_idx = nodes_arrangement_scheme[ny as usize][nx as usize];
                        sand_graph.add_edge(current_code_idx, neighbour_idx, 1);
                    } else {
                        sand_graph.add_edge(current_code_idx, SandGraph::SINK_NODE, 1);
                    }
                }
            }
        }


        SandPileModel {graph: sand_graph, embedding: EmbeddingToR3::new(embedding) }
    }
}

impl TriangleLattice {
    pub fn new() -> TriangleLattice {
        TriangleLattice {}
    }
}

impl Lattice2D for TriangleLattice {
    fn get_lattice_2d(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        let x_size = cuboid_hull[0];
        let y_size = cuboid_hull[1];

        let mut sand_graph = SandGraph::new();
        let mut embedding: Vec<math::Vec3d> = vec![[0.0, 0.0, 0.0]];

        let mut nodes_arrangement_scheme: Vec<Vec<NodeIndex>> = Vec::new();

        // with triangle side equal to 3^0.5, distance between triangles centers is equal to 1.0
        let triangle_side = (3.0_f64).powf(0.5);
        let X = triangle_side * (3.0_f64).powf(0.5) / 6.0;

        let mut iy: usize = 0;
        loop {
            let dy = match iy % 4 {
                0 => 0.0,
                1 => X,
                2 => 3.0*X,
                _ => 4.0*X
            };
            let y = 6.0*X*((iy / 4) as f64) + dy;

            if y > y_size {
                break
            }

            nodes_arrangement_scheme.push(Vec::new());

            let mut ix: usize = 0;
            loop {
                let dx = match iy % 4 {
                    0 => 0.0,
                    1 => 0.5*triangle_side,
                    2 => 0.5*triangle_side,
                    _ => 0.0
                };
                let x = triangle_side*(ix as f64) + dx;

                if x > x_size {
                    break
                }

                let node_idx = sand_graph.add_node();
                embedding.push([x, y, 0.0]);
                nodes_arrangement_scheme[iy].push(node_idx);
                ix += 1;
            }
            iy += 1;
        }

        let neighbours = [
            [(0, -1), (-1, 1), (0, 1)],
            [(0, -1), (1, -1), (0, 1)],
            [(0, -1), (1, 1), (0, 1)],
            [(-1, -1), (0, -1), (0, 1)]];

        for iy in 0..nodes_arrangement_scheme.len() {
            for ix in 0..nodes_arrangement_scheme[iy].len() {
                let current_node_idx = nodes_arrangement_scheme[iy][ix];

                for (dx, dy) in neighbours[iy % 4].iter() {
                    let nx = (ix as i32) + *dx;
                    let ny = (iy as i32) + *dy;

                    if 0 <= ny && ny < nodes_arrangement_scheme.len() as i32 &&
                        0 <= nx && nx < nodes_arrangement_scheme[ny as usize].len() as i32 {
                        let neighbour_idx = nodes_arrangement_scheme[ny as usize][nx as usize];
                        sand_graph.add_edge(current_node_idx, neighbour_idx, 1);
                    } else {
                        sand_graph.add_edge(current_node_idx, SandGraph::SINK_NODE, 1);
                    }
                }
            }
        }


        SandPileModel {graph: sand_graph, embedding: EmbeddingToR3::new(embedding) }
    }
}

