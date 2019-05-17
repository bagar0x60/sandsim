use model::SandPileModel;
use model::sand_graph::{NodeIndex, SandGraph, NodeData};
use graphics::math;

pub struct SandPileController<'a> {
    pipeline: Vec<&'a Fn(i32, i32) -> i32>,
    pub model: SandPileModel,
    stack: Vec<NodeIndex>,
    is_in_stack: Vec<bool>,
    need_update_stack: bool,
}

impl<'a> SandPileController<'a> {
    pub const TOPPLE_ROUNDS: usize = 10000000;

    pub fn new(model: SandPileModel) -> SandPileController<'a> {
        let stack = Vec::new();
        let mut is_in_stack = vec![false; model.graph.nodes.len()];
        let need_update_stack = true;
        is_in_stack[SandGraph::SINK_NODE] = true;
        let pipeline: Vec<&'a Fn(i32, i32) -> i32> = Vec::new();

        SandPileController { pipeline, model, stack, is_in_stack, need_update_stack}
    }

    pub fn set_pipeline(&mut self, pipeline: Vec<&'a Fn(i32, i32) -> i32>) {
        self.pipeline = pipeline;
        self.pipeline.reverse();
    }

    pub fn add_sand(&mut self, coords: math::Vec3d<f32>, sand_count: i32) {
        let node_idx = self.model.embedding.get_node_by_coords(coords);
        let sand = &self.model.graph.nodes[node_idx].sand;
        sand.set(sand.get() + sand_count);
    }

    pub fn add_sand_to_all_nodes(&mut self, addable_sand: i32) {
        let graph = &mut self.model.graph;
        for node_idx in graph.non_sink_nodes() {
            let sand = &graph.nodes[node_idx].sand;
            sand.set(sand.get() + addable_sand);
        }
        self.need_update_stack = true;
    }

    pub fn max_stable(&mut self) {
        let graph = &mut self.model.graph;
        for node_idx in graph.non_sink_nodes() {
            let sand = &graph.nodes[node_idx].sand;
            let degree = graph.nodes[node_idx].degree;
            sand.set(degree - 1);
        }
        self.need_update_stack = true;
    }

    pub fn clear_sand(&mut self) {
        let graph = &mut self.model.graph;
        for node_idx in graph.non_sink_nodes() {
            graph.nodes[node_idx].sand.set(0);
        }
        self.need_update_stack = true;
    }

    fn clear_stack(&mut self) {
        self.stack.clear();
        self.is_in_stack = vec![false; self.model.graph.nodes.len()];
        self.is_in_stack[SandGraph::SINK_NODE] = true;
        self.need_update_stack = true;
    }

    fn update_stack(&mut self) {
        self.clear_stack();

        let graph = & self.model.graph;
        for node_idx in graph.non_sink_nodes() {
            let node = &graph.nodes[node_idx];

            if node.sand.get() >= node.degree {
                self.stack.push(node_idx);
                self.is_in_stack[node_idx] = true;
            }
        }
        self.need_update_stack = false;
    }

    // todo do something with this abomination
    fn add_node_to_stack_if_needed(stack: &mut Vec<NodeIndex>, is_in_stack: &mut Vec<bool>, node: &NodeData, node_idx: NodeIndex) {
        if ! is_in_stack[node_idx] && node.sand.get() >= node.degree {
            stack.push(node_idx);
            is_in_stack[node_idx] = true;
        }
    }

    pub fn is_relaxed(&self) -> bool {
        (! self.need_update_stack) && self.stack.is_empty()
    }

    fn change_sand_by_function(&mut self, f: &Fn(i32, i32) -> i32) {
        for node_idx in self.model.graph.non_sink_nodes() {
            let current_sand = self.model.graph.nodes[node_idx].sand.get();
            let degree = self.model.graph.nodes[node_idx].degree;
            let new_sand = f(current_sand, degree);
            self.model.graph.nodes[node_idx].sand.set(new_sand);
        }
        self.need_update_stack = true;
    }

    pub fn update(&mut self) {
        if self.need_update_stack {
            self.update_stack();
        }

        if self.is_relaxed() {
            if let Some(func) = self.pipeline.pop() {
                self.change_sand_by_function(&func);
            }
            return;
        }

        let graph = & self.model.graph;
        for _ in 0..Self::TOPPLE_ROUNDS {
            if let Some(node_idx) = self.stack.pop() {
                let node = &graph.nodes[node_idx];
                for (weight, neighbour_node_idx) in graph.successors(node_idx) {
                    let neighbour = & graph.nodes[neighbour_node_idx];
                    neighbour.sand.set(neighbour.sand.get() + weight);

                    SandPileController::add_node_to_stack_if_needed(&mut self.stack, &mut self.is_in_stack, neighbour, neighbour_node_idx);
                }
                node.sand.set(node.sand.get() - node.degree);
                self.is_in_stack[node_idx] = false;

                SandPileController::add_node_to_stack_if_needed(&mut self.stack, &mut self.is_in_stack, node, node_idx);
            } else {
                break
            }
        }
    }


}