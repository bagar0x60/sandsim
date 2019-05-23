use super::utils::{UniformTiling, FigureGeometricInfo, FULL_CIRCLE, Constructor, continue_tiling_by_translation};
use super::Lattice;
use super::tilings::*;

use model::region::Cuboid;
use model::sand_graph::{SandGraph, NodeIndex};
use model::embedding::{EmbeddingToR3, Figure};
use model::SandPileModel;

#[derive(Debug)]
pub struct SemiRegularLattice {
    tiling_code: Vec<usize>,
}

pub struct KUniformLattice {

}

pub struct TetrahedralOctahedral {
}

impl SemiRegularLattice {
    pub fn new(tiling_code: Vec<usize>) -> Self {
        let angle: usize = tiling_code.iter().map(|n| FULL_CIRCLE / 2 - FULL_CIRCLE / n).sum();
        assert_eq!(angle % FULL_CIRCLE, 0,
                   "Incorrect tiling code {:#?}. Figures don't sum up to {} degrees", tiling_code, FULL_CIRCLE);

        SemiRegularLattice { tiling_code }
    }
}

impl Lattice for SemiRegularLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        let side_size = 1.0_f32;
        let [x_size, y_size, _] = *cuboid_hull;

        let mut tiling_builder = UniformTiling::new(1.0);

        // add figures from self.tiling_code around [0.0, 0.0, 0.0]
        let mut rotate = 0;
        for sides_count in &self.tiling_code {
            let figure =
                FigureGeometricInfo::new([x_size / 2.0, y_size / 2.0, 0.0], rotate, side_size, *sides_count);

            rotate = (rotate + figure.alpha) % FULL_CIRCLE;
            tiling_builder.add_figure(figure);
        }

        // add all other figures
        loop {
            let mut new_figures: Vec<FigureGeometricInfo> = Vec::new();

            for (pos, node_figures) in &mut tiling_builder.vertices_info.data {
                let [x, y, _] = *pos;
                if 0.0 <= x && x < x_size && 0.0 <= y && y < y_size {
                    let new_figures_partial_info = node_figures.new_figures(&self.tiling_code);

                    if new_figures_partial_info.len() > 0 {
                        new_figures = new_figures_partial_info
                            .iter()
                            .map(|(angle, sides_count)| FigureGeometricInfo::new(*pos, *angle, side_size, *sides_count))
                            .collect();
                        break
                    }
                }
            }

            if new_figures.len() > 0 {
                for figure in new_figures {
                    tiling_builder.add_figure(figure);
                }
            } else {
                break
            }
        }


        tiling_builder.build()
    }
}




impl KUniformLattice {
    pub fn new() -> Self {
        KUniformLattice {}
    }
}

impl Lattice for KUniformLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        tiling_14(cuboid_hull, 0)
    }
}


impl TetrahedralOctahedral {
    pub fn new() -> Self {
        TetrahedralOctahedral {}
    }

    fn tetrahedron1() -> Figure {
        let vertices = vec![
            [0.0, 0.5, -2.0_f32.powf(0.5)/4.0],
            [0.0, -0.5, -2.0_f32.powf(0.5)/4.0],
            [0.5, 0.0, 2.0_f32.powf(0.5)/4.0],
            [-0.5, 0.0, 2.0_f32.powf(0.5)/4.0],
        ];
        let indexes = vec![0, 1, 2, 0, 2, 3, 0, 1, 3, 1, 2, 3];
        let border_indexes = vec![0, 1, 0, 2, 0, 3, 1, 2, 1, 3, 2, 3];
        Figure {vertices, indexes, border_indexes}
    }

    fn tetrahedron2() -> Figure {
        let vertices = vec![
            [0.5, 0.0, -2.0_f32.powf(0.5)/4.0],
            [-0.5, 0.0, -2.0_f32.powf(0.5)/4.0],
            [0.0, 0.5, 2.0_f32.powf(0.5)/4.0],
            [0.0, -0.5, 2.0_f32.powf(0.5)/4.0],
        ];
        let indexes = vec![0, 1, 2, 0, 2, 3, 0, 1, 3, 1, 2, 3];
        let border_indexes = vec![0, 1, 0, 2, 0, 3, 1, 2, 1, 3, 2, 3];
        Figure {vertices, indexes, border_indexes}
    }

    fn octahedron() -> Figure {
        let sqrt2 = 2.0_f32.powf(0.5);
        let vertices = vec![
            [0.5, 0.5, 0.0], [0.5, -0.5, 0.0],
            [-0.5, -0.5, 0.0], [-0.5, 0.5, 0.0],
            [0.0, 0.0, -sqrt2/2.0], [0.0, 0.0, sqrt2/2.0],
        ];
        let indexes = vec![
            4, 0, 1,  4, 1, 2,  4, 2, 3,  4, 3, 0,
            5, 0, 1,  5, 1, 2,  5, 2, 3,  5, 3, 0,
        ];
        let border_indexes = vec![
            4, 0,  4, 1,  4, 2,  4, 3,
            5, 0,  5, 1,  5, 2,  5, 3,
            0, 1,  1, 2,  2, 3,  3, 0,
        ];
        Figure {vertices, indexes, border_indexes}
    }

    fn empty() -> Figure {
        let vertices = vec![];
        let indexes = vec![];
        let border_indexes = vec![];
        Figure {vertices, indexes, border_indexes}
    }
}

impl Lattice for TetrahedralOctahedral {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        let x_size_even = cuboid_hull[0] as i32;
        let y_size_even = cuboid_hull[1] as i32;
        let z_size_even = (cuboid_hull[2] / 2.0_f32.powf(0.5)) as i32;
        let even_count = x_size_even*y_size_even*z_size_even;

        let coords_to_index_even =
            |x: i32, y: i32, z: i32| { z*x_size_even*y_size_even + y*x_size_even + x + 1};
        let real_coords_even =
            |x: i32, y: i32, z: i32| { [x as f32, y as f32, 2.0_f32.powf(0.5)*(z as f32)] };

        let x_size_odd = (cuboid_hull[0] - 0.5) as i32;
        let y_size_odd = (cuboid_hull[1] - 0.5) as i32;
        let z_size_odd = (cuboid_hull[2] / 2.0_f32.powf(0.5) - 0.5) as i32;
        let odd_count = x_size_odd*y_size_odd*z_size_odd;

        let coords_to_index_odd =
            |x: i32, y: i32, z: i32| { z*x_size_odd*y_size_odd + y*x_size_odd + x + even_count + 1};
        let real_coords_odd =
            |x: i32, y: i32, z: i32| { [x as f32 + 0.5, y as f32 + 0.5, 2.0_f32.powf(0.5)*(z as f32 + 0.5)] };

        let mut sand_graph = SandGraph::new();
        let mut embedding = EmbeddingToR3::new();

        embedding.add_figure(Self::octahedron());
        embedding.add_figure(Self::tetrahedron1());
        embedding.add_figure(Self::tetrahedron2());

        // add octahedral
        for z in 0..z_size_even {
            for y in 0..y_size_even {
                for x in 0..x_size_even {
                    let idx = sand_graph.add_node();
                    let coords = real_coords_even(x, y, z);
                    embedding.set_node_info(idx, coords, 0);
                    assert_eq!(idx, coords_to_index_even(x, y, z) as usize);
                }
            }
        }
        for z in 0..z_size_odd {
            for y in 0..y_size_odd {
                for x in 0..x_size_odd {
                    let idx = sand_graph.add_node();
                    let coords = real_coords_odd(x, y, z);
                    embedding.set_node_info(idx, coords, 0);
                    assert_eq!(idx, coords_to_index_odd(x, y, z) as usize);
                }
            }
        }

        let mean4 = |a, b, c, d| {
            vecmath::vec3_scale(
                vecmath::vec3_add(
                    vecmath::vec3_add(a, b),
                    vecmath::vec3_add(c, d),
                ), 0.25)
        };

        let inside_hull = |v: [f32; 3]| {
            0.0 <= v[0] && v[0] < cuboid_hull[0] &&
                0.0 <= v[1] && v[1] < cuboid_hull[1] &&
                0.0 <= v[2] && v[2] < cuboid_hull[2]
        };

        let inside_even = |x, y, z| {
            0 <= x && x < x_size_even &&
                0 <= y && y < y_size_even &&
                0 <= z && z < z_size_even
        };

        let inside_odd = |x, y, z| {
            0 <= x && x < x_size_odd &&
                0 <= y && y < y_size_odd &&
                0 <= z && z < z_size_odd
        };

        // add tetrahedral and edges
        let neighbours = [
            [(0, 0, 0), (1, 0, 0), (0, 0, 0), (0, -1, 0)],
            [(0, 0, 0), (1, 0, 0), (0, 0, -1), (0, -1, -1)],
            [(0, 0, 0), (0, 1, 0), (0, 0, -1), (-1, 0, -1)],
            [(0, 0, 0), (0, 1, 0), (0, 0, 0), (-1, 0, 0)],
        ];

        for z in -1..z_size_even + 1 {
            for y in -1..y_size_even + 1 {
                for x in -1..x_size_even + 1 {
                    for (i, neighbour) in neighbours.iter().enumerate() {
                        let [n1, n2, n3, n4] = neighbour;
                        let coords_1 = real_coords_even(x + n1.0, y + n1.1, z + n1.2);
                        let coords_2 = real_coords_even(x + n2.0, y + n2.1, z + n2.2);

                        let coords_3 = real_coords_odd(x + n3.0, y + n3.1, z + n3.2);
                        let coords_4 = real_coords_odd(x + n4.0, y + n4.1, z + n4.2);

                        let coords = mean4(coords_1, coords_2, coords_3, coords_4);

                        if inside_hull(coords) {
                            let idx = sand_graph.add_node();
                            embedding.set_node_info(idx, coords, 1 + (i % 2));

                            for (j, n) in neighbour.iter().enumerate() {
                                let [xn, yn, zn] = [x + n.0, y + n.1, z + n.2];
                                if j < 2 && inside_even(xn, yn, zn) {
                                    let n_idx = coords_to_index_even(xn, yn, zn) as usize;
                                    sand_graph.add_edge(idx, n_idx, 1);
                                    sand_graph.add_edge(n_idx, idx, 1);
                                } else if inside_odd(xn, yn, zn) {
                                    let n_idx = coords_to_index_odd(xn, yn, zn) as usize;
                                    sand_graph.add_edge(idx, n_idx, 1);
                                    sand_graph.add_edge(n_idx, idx, 1);
                                }
                            }
                        }
                    }

                }
            }
        }

        // add edges to sink node
        for idx in 1..(even_count + odd_count + 1) as usize {
            if sand_graph.nodes[idx].degree != 8 {
                let d = sand_graph.nodes[idx].degree;
                sand_graph.add_edge(idx, SandGraph::SINK_NODE, 8 - d);
            }
        }
        for idx in (even_count + odd_count + 1) as usize..sand_graph.nodes.len() {
            if sand_graph.nodes[idx].degree != 4 {
                let d = sand_graph.nodes[idx].degree;
                sand_graph.add_edge(idx, SandGraph::SINK_NODE, 4 - d);
            }
        }

        // println!("{:#?}", sand_graph);

        SandPileModel { graph: sand_graph, embedding}
    }
}