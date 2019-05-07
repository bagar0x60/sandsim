use super::Lattice;

use graphics::math;
use model::SandPileModel;
use model::region::Cuboid;
use model::sand_graph::{SandGraph, NodeIndex};
use model::embedding::{ EmbeddingToR3, Figure };


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
        let x_size = cuboid_hull[0] as usize;
        let y_size = cuboid_hull[1] as usize;
        let edges_count = x_size*y_size;

        let coords_to_index = |x: usize, y: usize| -> NodeIndex { y*x_size + x + 1};

        let mut sand_graph = SandGraph::new();

        for _ in 0..edges_count {
            sand_graph.add_node();
        }

        let mut neighbours: [NodeIndex; 4];
        let mut embedding = EmbeddingToR3::new();

        let radius = (0.5_f32).powf(0.5);
        let figure = Figure::polygon_on_circle(radius, 4, 45.0, 90.0);
        embedding.add_figure(figure);


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

                embedding.set_node_info(this_node, [x as f32, y as f32, 0.0], 0);
            }
        }
        SandPileModel {graph: sand_graph, embedding }
    }
}

impl HexagonLattice {
    pub fn new() -> HexagonLattice {
        HexagonLattice {}
    }
}

impl Lattice for HexagonLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        let x_size = cuboid_hull[0];
        let y_size = cuboid_hull[1];

        let mut sand_graph = SandGraph::new();
        let mut embedding = EmbeddingToR3::new();

        let radius = 1.0 / (3_f32).powf(0.5);
        let figure = Figure::polygon_on_circle(radius, 6, 30.0, 60.0);
        embedding.add_figure(figure);

        let mut nodes_arrangement_scheme: Vec<Vec<NodeIndex>> = Vec::new();

        let mut iy: usize = 0;
        loop {
            let y = (0.75_f32).powf(0.5) * (iy as f32);
            if y > y_size {
                break
            }

            nodes_arrangement_scheme.push(Vec::new());

            let mut ix: usize = 0;
            loop {
                let x = (ix as f32) + 0.5*((iy % 2) as f32);
                if x > x_size {
                    break
                }

                let node_idx = sand_graph.add_node();
                embedding.set_node_info(node_idx, [x, y, 0.0], 0);
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


        SandPileModel {graph: sand_graph, embedding }
    }
}

impl TriangleLattice {
    pub fn new() -> TriangleLattice {
        TriangleLattice {}
    }
}

impl Lattice for TriangleLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        let x_size = cuboid_hull[0];
        let y_size = cuboid_hull[1];

        // with triangle side equal to 3^0.5, distance between triangles centers is equal to 1.0
        let triangle_side = (3.0_f32).powf(0.5);
        let X = triangle_side * (3.0_f32).powf(0.5) / 6.0;

        let mut sand_graph = SandGraph::new();
        let mut embedding = EmbeddingToR3::new();

        let radius = 2.0*X;
        let figure1 = Figure::polygon_on_circle(radius, 3, 90.0, 120.0);
        let figure2 = Figure::polygon_on_circle(radius, 3, 30.0, 120.0);
        embedding.add_figure(figure1);
        embedding.add_figure(figure2);

        let mut nodes_arrangement_scheme: Vec<Vec<NodeIndex>> = Vec::new();



        let mut iy: usize = 0;
        loop {
            let dy = match iy % 4 {
                0 => 0.0,
                1 => X,
                2 => 3.0*X,
                _ => 4.0*X
            };
            let y = 6.0*X*((iy / 4) as f32) + dy;

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
                let x = triangle_side*(ix as f32) + dx;

                if x > x_size {
                    break
                }

                let node_idx = sand_graph.add_node();
                let figure_idx = iy % 2;
                embedding.set_node_info(node_idx, [x, y, 0.0], figure_idx);
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


        SandPileModel {graph: sand_graph, embedding }
    }
}

impl CubeLattice {
    pub fn new() -> Self {
        CubeLattice {}
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
        let figure = Figure { vertices, indexes };
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