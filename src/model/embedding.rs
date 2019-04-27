extern crate graphics;

use super::sand_graph::{NodeIndex};
use self::graphics::math;

pub struct EmbeddingToR3 {
    nodes_coordinates: Vec<math::Vec3d<f32>>,
    nodes_figures: Vec<usize>,
    pub unique_figures: Vec<Figure>,
}

#[derive(Clone)]
pub struct Figure {
    pub vertices: Vec<math::Vec3d<f32>>,
    pub indexes: Vec<usize>,
}

impl EmbeddingToR3 {
    pub fn new() -> EmbeddingToR3 {
        EmbeddingToR3 {
            nodes_coordinates: Vec::new(),
            nodes_figures: Vec::new(),
            unique_figures: Vec::new()
        }
    }

    pub fn add_figure(&mut self, figure: Figure) -> usize {
        self.unique_figures.push(figure);
        self.unique_figures.len() - 1
    }

    pub fn set_node_info(&mut self, node_index: NodeIndex, coords: math::Vec3d<f32>, figure_index: usize) {
        if self.nodes_coordinates.len() <= node_index {
            let new_elems_count = node_index - self.nodes_coordinates.len() + 1;
            self.nodes_coordinates.append(&mut vec![[0.0; 3]; new_elems_count]);
            self.nodes_figures.append(&mut vec![0 as usize; new_elems_count]);
        }
        self.nodes_coordinates[node_index] = coords;
        self.nodes_figures[node_index] = figure_index;
    }

    pub fn get_node_info(&self, node_index: NodeIndex) -> (math::Vec3d<f32>, usize) {
        (self.nodes_coordinates[node_index].clone(), self.nodes_figures[node_index])
    }

    pub fn get_node_by_coords(&self, coords: math::Vec3d<f32>) -> NodeIndex {
        let mut min_distance = std::f32::MAX;
        let mut min_node_idx = 0;
        for (idx, point) in self.nodes_coordinates.iter().enumerate().skip(1) {
            let distance = (coords[0] - point[0]).powi(2)
                + (coords[1] - point[1]).powi(2)
                + (coords[2] - point[2]).powi(2);
            if distance < min_distance {
                min_node_idx = idx;
                min_distance = distance;
            }
        }

        min_node_idx
    }



    /*
    pub fn swap_xy(&self) -> Self {
        let mut new_nodes_coordinates: Vec<math::Vec3d> = Vec::new();
        for [x, y, z] in &self.nodes_coordinates {
            new_nodes_coordinates.push([*y, *x, *z]);
        }
        EmbeddingToR3 { nodes_coordinates: new_nodes_coordinates }
    }
    */
}