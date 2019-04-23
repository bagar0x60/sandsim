extern crate graphics;

use super::sand_graph::{NodeIndex};
use self::graphics::math;

pub struct EmbeddingToR3 {
    nodes_coordinates: Vec<math::Vec3d>,
}

impl EmbeddingToR3 {
    pub fn new(nodes_coordinates: Vec<math::Vec3d>) -> EmbeddingToR3 {
        EmbeddingToR3 {nodes_coordinates}
    }

    pub fn node_to_coordinates(&self, node: NodeIndex) -> math::Vec3d {
        self.nodes_coordinates[node].clone()
    }

    pub fn coords_to_node(&self, coords: math::Vec3d) -> NodeIndex {
        let mut min_distance = std::f64::MAX;
        let mut min_node_idx = 0;
        for (idx, point) in self.nodes_coordinates.iter().enumerate().skip(1) {
            let distance = (coords[0] - point[0]).powi(2) + (coords[1] - point[1]).powi(2) + (coords[2] - point[2]).powi(2);
            if distance < min_distance {
                min_node_idx = idx;
                min_distance = distance;
            }
        }

        min_node_idx
    }

    pub fn swap_xy(&self) -> Self {
        let mut new_nodes_coordinates: Vec<math::Vec3d> = Vec::new();
        for [x, y, z] in &self.nodes_coordinates {
            new_nodes_coordinates.push([*y, *x, *z]);
        }
        EmbeddingToR3 { nodes_coordinates: new_nodes_coordinates }
    }
}