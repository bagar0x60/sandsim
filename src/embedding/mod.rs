extern crate graphics;

use sand_graph::{NodeIndex};
use self::graphics::math;

pub struct EmbeddingToR3 {
    nodes_coordinates: Vec<math::Vec3d>,
}

impl EmbeddingToR3 {
    pub fn new(nodes_coordinates: Vec<math::Vec3d>) -> EmbeddingToR3 {
        EmbeddingToR3 {nodes_coordinates}
    }

    pub fn node_to_coordinates(&self, node: NodeIndex) -> math::Vec3d {
        self.nodes_coordinates[node]
    }
}