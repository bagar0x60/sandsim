extern crate graphics;

use embedding::EmbeddingToR3;
use self::graphics::math;

pub type NodeIndex = usize;
pub type EdgeIndex = usize;

pub struct NodeData {
    sand: i32,
    first_outgoing_edge: Option<EdgeIndex>
}

pub struct EdgeData {
    weight: u32,
    target: NodeIndex,
    next_outgoing_edge: Option<EdgeIndex>
}

pub struct SandGraph {
    nodes: Vec<NodeData>,
    edges: Vec<EdgeData>,
}

impl SandGraph {
    const SINK_NODE: NodeIndex = 0;

    pub fn new() -> SandGraph {
        let sink = NodeData {sand: 0, first_outgoing_edge: None};
        SandGraph {nodes: vec![sink], edges: vec![]}
    }

    pub fn add_node(&mut self) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(NodeData { sand: 0, first_outgoing_edge: None});
        index
    }

    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, weight: u32) {
        let edge_index = self.edges.len();
        let node_data = &mut self.nodes[source];
        self.edges.push(EdgeData {
            weight: weight,
            target: target,
            next_outgoing_edge: node_data.first_outgoing_edge
        });
        node_data.first_outgoing_edge = Some(edge_index);
    }
}

pub fn square(edge_size: usize) -> (SandGraph, EmbeddingToR3) {
    let coords_to_index = |x: usize, y: usize| -> NodeIndex { y*edge_size + x + 1};

    let mut sand_graph = SandGraph::new();
    for i in 1..edge_size.pow(2) {
        sand_graph.add_node();
    }

    let mut neighbours: [NodeIndex; 4] = [0, 0, 0, 0];
    let mut embedding: Vec<math::Vec3d> = vec![[0.0, 0.0, 0.0]; edge_size.pow(2) + 1];

    for x in 0..edge_size {
        for y in 0..edge_size {
            if x > 0 { neighbours[0] = coords_to_index(x - 1, y) } else {neighbours[0] = 0};
            if x < edge_size - 1 { neighbours[1] = coords_to_index(x + 1, y) } else {neighbours[1] = 0};
            if y > 0 { neighbours[2] = coords_to_index(x, y - 1) } else {neighbours[2] = 0};
            if x < edge_size - 1 { neighbours[3] = coords_to_index(x, y + 1) } else {neighbours[3] = 0};

            let this_node = coords_to_index(x, y);
            for neighbour in &neighbours {
                sand_graph.add_edge(this_node, *neighbour, 1);
            }

            embedding[this_node] = [x as f64, y as f64, 0.0];
        }
    }

    ( sand_graph, EmbeddingToR3::new(embedding) )
}