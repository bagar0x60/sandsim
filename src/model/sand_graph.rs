pub type NodeIndex = usize;
pub type EdgeIndex = usize;

use std::cell::Cell;


#[derive(Debug)]
pub struct NodeData {
    pub sand: Cell<i32>,
    pub degree: i32,
    first_outgoing_edge: Option<EdgeIndex>
}


#[derive(Debug)]
pub struct EdgeData {
    weight: i32,
    target: NodeIndex,
    next_outgoing_edge: Option<EdgeIndex>
}

#[derive(Debug)]
pub struct SandGraph {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
}

pub struct Successors<'graph> {
    graph: &'graph SandGraph,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph> Iterator for Successors<'graph> {
    type Item = (i32, NodeIndex);

    fn next(&mut self) -> Option<(i32, NodeIndex)> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some((edge.weight, edge.target))
            }
        }
    }
}


impl SandGraph {
    pub const SINK_NODE: NodeIndex = 0;

    pub fn new() -> SandGraph {
        let sink = NodeData {sand: Cell::new(0), degree: 0, first_outgoing_edge: None};
        SandGraph {nodes: vec![sink], edges: vec![]}
    }

    pub fn add_node(&mut self) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(NodeData { sand: Cell::new(0), degree: 0, first_outgoing_edge: None});
        index
    }

    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, weight: i32) {
        let edge_index = self.edges.len();
        let node_data = &mut self.nodes[source];
        self.edges.push(EdgeData {
            weight: weight,
            target: target,
            next_outgoing_edge: node_data.first_outgoing_edge
        });
        node_data.first_outgoing_edge = Some(edge_index);
        node_data.degree += weight;
    }

    pub fn successors(&self, source: NodeIndex) -> Successors {
        let first_outgoing_edge = self.nodes[source].first_outgoing_edge;
        Successors { graph: self, current_edge_index: first_outgoing_edge }
    }


    pub fn non_sink_nodes(&self) -> impl Iterator<Item = NodeIndex> {
        1 .. self.nodes.len()
    }
}




