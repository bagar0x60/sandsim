pub mod embedding;
pub mod sand_graph;
pub mod lattice;
pub mod region;

use self::embedding::EmbeddingToR3;
use self::sand_graph::{SandGraph};
use self::region::Region;
use self::lattice::Lattice;

#[derive(Debug)]
pub struct SandPileModel {
    pub graph: SandGraph,
    pub embedding: EmbeddingToR3,
}

impl SandPileModel {
    pub fn new<L: Lattice, R: Region>(region: R, lattice: L) -> SandPileModel {
        let mut new_graph = SandGraph::new();
        let mut new_embedding = EmbeddingToR3::new();

        let cuboid_hull = region.cuboid_hull();
        let lattice_inside_hull = lattice.get_lattice(&cuboid_hull);
        let old_graph: SandGraph = lattice_inside_hull.graph;
        let old_embedding: EmbeddingToR3 = lattice_inside_hull.embedding;

        let mut old_to_new_idx_map = vec![SandGraph::SINK_NODE; old_graph.nodes.len()];

        let mut node_counter: usize = 0;
        for node_idx in old_graph.non_sink_nodes() {
            let (coords, figure_idx) = old_embedding.get_node_info(node_idx);
            if region.is_point_inside_region(&coords) {
                node_counter += 1;
                old_to_new_idx_map[node_idx] = node_counter;
                new_graph.add_node();
                new_embedding.set_node_info(node_counter, coords, figure_idx);
            }
        }

        for old_node_idx in old_graph.non_sink_nodes() {
            let new_node_idx = old_to_new_idx_map[old_node_idx];
            if new_node_idx == SandGraph::SINK_NODE {
                continue;
            }
            for (weight, neighbour_old_idx) in old_graph.successors(old_node_idx) {
                let neighbour_new_idx = old_to_new_idx_map[neighbour_old_idx];
                new_graph.add_edge(new_node_idx, neighbour_new_idx, weight);
            }
        }

        new_embedding.unique_figures = old_embedding.unique_figures;

        SandPileModel {graph: new_graph, embedding: new_embedding }
    }

    pub fn transpose(&mut self) {
        use graphics::math;

        for node_idx in self.graph.non_sink_nodes() {
            let ([x, y, z], figure_idx) = self.embedding.get_node_info(node_idx);
            self.embedding.set_node_info(node_idx, [y, x, z], figure_idx);
        }

        for figure_idx in 0..self.embedding.unique_figures.len() {
            let figure = &mut self.embedding.unique_figures[figure_idx];
            let new_vertices: Vec<math::Vec3d<f32>> =
                figure.vertices.iter().map(|[x, y, z]| [*y, *x, *z]).collect();
            figure.vertices = new_vertices;
        }
    }
}