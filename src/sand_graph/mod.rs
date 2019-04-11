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
        let mut sink = NodeData {sand: 0, first_outgoing_edge: None};
        SandGraph {nodes: vec![sink], edges: vec![]}
    }

    pub fn add_node(&mut self, sand: i32) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(NodeData { sand: sand, first_outgoing_edge: None});
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