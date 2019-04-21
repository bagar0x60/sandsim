use model::SandPileModel;

pub struct SandPileController {
    pub model: SandPileModel,
}

impl SandPileController {
    pub fn new(model: SandPileModel) -> SandPileController{
        SandPileController {model}
    }

    pub fn add_sand_to_all_nodes(&mut self, addable_sand: i32) {
        let graph = &mut self.model.graph;
        for node_idx in graph.non_sink_nodes() {
            let sand = &graph.nodes[node_idx].sand;
            sand.set(sand.get() + addable_sand);
        }
    }

    pub fn max_stable(&mut self) {
        let graph = &mut self.model.graph;
        for node_idx in graph.non_sink_nodes() {
            let sand = &graph.nodes[node_idx].sand;
            let degree = graph.nodes[node_idx].degree;
            sand.set(degree - 1);
        }
    }

    pub fn clear_sand(&mut self) {
        let graph = &mut self.model.graph;
        for node_idx in graph.non_sink_nodes() {
            graph.nodes[node_idx].sand.set(0);
        }
    }

    pub fn topple(&mut self) {
        let graph = & self.model.graph;
        for node_idx in graph.non_sink_nodes() {
            let node = &graph.nodes[node_idx];

            if node.sand.get() >= node.degree {
                for (weight, neighbour_node_idx) in graph.successors(node_idx) {
                    let neighbour_sand = & graph.nodes[neighbour_node_idx].sand;
                    neighbour_sand.set(neighbour_sand.get() + weight);
                }
                node.sand.set(node.sand.get() - node.degree);
            }


        }
    }


}