pub const FULL_CIRCLE: usize = 720;

use graphics::math;
use model::SandPileModel;
use model::region::Cuboid;
use model::sand_graph::{SandGraph, NodeIndex};
use model::embedding::{ EmbeddingToR3, Figure };
use std::collections::linked_list::LinkedList;
use vecmath;
use graphics::math::Vec3d;

#[derive(Clone, Copy, Debug)]
pub(super) struct FigureVertexInfo {
    pub sides_count: usize,
    pub figure_idx: NodeIndex,
    pub angle: usize,
}

#[derive(Debug)]
pub(super) struct FigureGeometricInfo {
    pub alpha: usize,
    pub beta: usize,
    pub r: f32,
    pub center: math::Vec3d<f32>,
    pub rotate: usize,
    pub side_size: f32,
    pub sides_count: usize,
}

#[derive(Debug)]
pub(super) struct VertexFigures {
    pub figures: Vec<FigureVertexInfo>,
    check_for_new_figures: bool,
}

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::TrySendError::Full;

#[derive(Debug)]
pub(super) struct UniformTilingBuilder {
    pub figures: Vec<FigureGeometricInfo>,
    pub vertices_info: Vec<(math::Vec3d<f32>, VertexFigures)>,
}

use regex::Regex;
impl VertexFigures {
    pub fn new() -> Self {
        VertexFigures { figures: Vec::new(), check_for_new_figures: true }
    }

    fn is_complete(&self) -> bool {
        let mut angles_sum = 0;

        for figure in &self.figures {
            angles_sum += FULL_CIRCLE / 2 - FULL_CIRCLE / figure.sides_count;
        }

        angles_sum == FULL_CIRCLE
    }

    pub fn add(&mut self, figure: FigureVertexInfo) {
        self.check_for_new_figures = true;

        for i in 0..self.figures.len() {
            if figure.angle < self.figures[i].angle {
                self.figures.insert(i, figure);
                return
            }
        }
        let end_idx = self.figures.len();
        self.figures.insert(end_idx, figure);
    }

    fn code_to_str(tiling_code: &Vec<usize>) -> String {
        let tiling_code_strs: Vec<String> = tiling_code.iter().map(|n| n.to_string()).collect();
        tiling_code_strs[..].join(" ") + " "
    }

    fn get_regex(&self) -> Regex {

        let mut regex_str = "^".to_string();

        for i1 in 0..self.figures.len() {
            regex_str += &format!("{} ", self.figures[i1].sides_count);
            let i2 = (i1 + 1) % self.figures.len();
            let alpha_i1 = FULL_CIRCLE / 2 - FULL_CIRCLE / self.figures[i1].sides_count;
            if (self.figures[i1].angle + alpha_i1) % FULL_CIRCLE != self.figures[i2].angle % FULL_CIRCLE {
                regex_str += ".+";
            }
        }
        regex_str += "$";

        //todo println!("{:#?}\n regex: {}", self, regex_str);

        Regex::new(&regex_str).unwrap()
    }

    pub fn new_figures(&mut self, tiling_code: &Vec<usize>) -> Vec<(usize, usize)> {
        if self.figures.len() == tiling_code.len() || ! self.check_for_new_figures {
            return Vec::new();
        }

        self.check_for_new_figures = false;

        let mut tiling_code = tiling_code.clone();
        let regex = self.get_regex();

        let mut success_tiling_code: Option<Vec<usize>> = None;
        let mut success_tiling_code_str: Option<String> = None;

        for j in 0..2 {
            for i in 0..tiling_code.len() {
                let code_str = Self::code_to_str(&tiling_code);
                if regex.is_match(&code_str) {
                    if success_tiling_code_str == None {
                        success_tiling_code = Some(tiling_code.clone());
                        success_tiling_code_str = Some(code_str.clone());
                    }

                    if let Some(other_code_str) = &success_tiling_code_str {
                        if *other_code_str != code_str {
                            // println!("mask: {} code: {} other code: {}", regex.as_str(), code_str, other_code_str);
                            return Vec::new()
                        }
                    }
                }

                // cycling shift:  tiling_code >> 1
                let code = tiling_code.pop().unwrap();
                tiling_code.insert(0, code);
            }

            tiling_code.reverse();
        }

        // println!("current figures\n {:#?}", self.figures);
        // println!("mask\n {:#?}", tiling_code);


        if let Some(tiling_code) = success_tiling_code {
            let mut result: Vec<(usize, usize)> = Vec::new();

            let mut angle = self.figures[0].angle;

            let mut j = 0;


            for i in 0..tiling_code.len() {
                let alpha = FULL_CIRCLE / 2 - FULL_CIRCLE / tiling_code[i];
                if j < self.figures.len() && angle == self.figures[j].angle {
                    j += 1;
                } else {
                    let figure = (angle, tiling_code[i]);
                    result.push(figure);
                }
                angle += alpha;
            }
            return result;
        } else {
            panic!("Impossible to sustain tiling in this vertex {}", regex.as_str());
        }

        Vec::new()
    }
}


fn to_radians(angle: usize) -> f32 {
    (360.0 * (angle as f32) / (FULL_CIRCLE as f32)).to_radians()
}

fn pretty_close(v1: Vec3d<f32>, v2: Vec3d<f32>) -> bool {
    vecmath::vec3_len(vecmath::vec3_sub(v1, v2)) < 0.001
}

impl FigureGeometricInfo {
    pub fn new(position: math::Vec3d<f32>, rotate: usize, side_size: f32, sides_count: usize) -> Self {
        let rotate = rotate % FULL_CIRCLE;
        let alpha = (FULL_CIRCLE / 2 - FULL_CIRCLE / sides_count) % FULL_CIRCLE;
        let r = side_size / (2.0 * (to_radians(alpha)  / 2.0).cos() );
        let gamma = rotate + alpha / 2;
        let gamma_rad = to_radians(gamma);
        let center = vecmath::vec3_add([r*gamma_rad.cos(), r*gamma_rad.sin(), 0.0], position);
        FigureGeometricInfo { center, rotate, sides_count, side_size, r, alpha, beta: FULL_CIRCLE / 2 - alpha }
    }

    pub fn vertices(&self) -> Vec<(math::Vec3d<f32>, usize)> {
        let mut result: Vec<(math::Vec3d<f32>, usize)> = Vec::new();

        let mut gamma = (self.rotate + self.alpha / 2 + FULL_CIRCLE / 2)  % FULL_CIRCLE;
        let mut rotate = self.rotate;
        for i in 0..self.sides_count {
            let gamma_rad = to_radians(gamma);
            let position =  vecmath::vec3_add(
                self.center,
                [self.r*gamma_rad.cos(), self.r*gamma_rad.sin(), 0.0]
            );

            result.push((position, rotate));

            gamma = (gamma + self.beta) % FULL_CIRCLE;
            rotate = (rotate + self.beta) % FULL_CIRCLE;
        }

        result
    }
}


fn add_figure(nodes_info: &mut Vec<(math::Vec3d<f32>, VertexFigures)>,
              figure: FigureGeometricInfo,
              sand_graph: &mut SandGraph,
              embedding: &mut EmbeddingToR3,
              unique_figures: &mut HashMap<(usize, usize), usize>) {

    let graph_node_idx = sand_graph.add_node();

    let figure_key = (figure.sides_count, figure.rotate % (FULL_CIRCLE / figure.sides_count));
    let mut figure_index = 0;
    if ! unique_figures.contains_key(&figure_key) {
        let gamma = figure.rotate + figure.alpha / 2 + FULL_CIRCLE / 2;
        let figure_polygon =
            Figure::polygon_on_circle(figure.r, figure.sides_count,
                                      360.0*(gamma as f32) / (FULL_CIRCLE as f32),
                                      360.0*(figure.beta as f32) / (FULL_CIRCLE as f32));
        figure_index = embedding.add_figure(figure_polygon);
        unique_figures.insert(figure_key, figure_index);
    } else {
        figure_index = *unique_figures.get(&figure_key).unwrap();
    }

    embedding.set_node_info(graph_node_idx, figure.center, figure_index);

    let pretty_close = |v1: Vec3d<f32>, v2: Vec3d<f32>| {
        vecmath::vec3_len(vecmath::vec3_sub(v1, v2)) < 0.001
    };


    for (pos1, rotate) in figure.vertices() {
        let figure_node_info = FigureVertexInfo { angle: rotate, sides_count: figure.sides_count, figure_idx: graph_node_idx};
        let mut is_vertex_exist = false;

        for (pos2, nodes_figures) in nodes_info.iter_mut() {
            if pretty_close(pos1, *pos2) {
                nodes_figures.add(figure_node_info);
                is_vertex_exist = true;
                break;
            }
        }

        if ! is_vertex_exist {
            let mut node_figures = VertexFigures::new();
            node_figures.add(figure_node_info);
            nodes_info.push((pos1, node_figures));
        }
    }
}


impl UniformTilingBuilder {
    pub fn new() -> Self {
        let vertices_info: Vec<(math::Vec3d<f32>, VertexFigures)> = Vec::new();
        let figures: Vec<FigureGeometricInfo> = Vec::new();

        UniformTilingBuilder { vertices_info, figures }
    }

    pub fn add_figure(&mut self, figure: FigureGeometricInfo) {
        let figure_idx = self.figures.len();

        for (pos1, rotate) in figure.vertices() {
            let figure_node_info = FigureVertexInfo { angle: rotate, sides_count: figure.sides_count, figure_idx};
            let mut is_vertex_exist = false;

            for (pos2, nodes_figures) in self.vertices_info.iter_mut() {
                if pretty_close(pos1, *pos2) {
                    nodes_figures.add(figure_node_info);
                    is_vertex_exist = true;
                    break;
                }
            }

            if ! is_vertex_exist {
                let mut vertex_figures = VertexFigures::new();
                vertex_figures.add(figure_node_info);
                self.vertices_info.push((pos1, vertex_figures));
            }
        }

        self.figures.push(figure);
    }

    pub fn build(&self) -> SandPileModel {
        let mut sand_graph = SandGraph::new();
        let mut embedding = EmbeddingToR3::new();

        // add nodes to graph
        let mut figure_idx_to_node_idx: Vec<NodeIndex> = Vec::new();

        for _ in 0..self.figures.len() {
            let node_idx = sand_graph.add_node();
            figure_idx_to_node_idx.push(node_idx);
        }

        // add nodes coords to embedding
        // add figures to embedding
        let mut unique_figures_polygons: HashMap<(usize, usize), usize> = HashMap::new();

        for (figure_idx, figure) in self.figures.iter().enumerate() {
            let figure_key = (figure.sides_count, figure.rotate % (FULL_CIRCLE / figure.sides_count));

            let mut figure_polygon_idx = 0;
            if ! unique_figures_polygons.contains_key(&figure_key) {
                let gamma = figure.rotate + figure.alpha / 2 + FULL_CIRCLE / 2;
                let figure_polygon =
                    Figure::polygon_on_circle(figure.r, figure.sides_count,
                                              360.0*(gamma as f32) / (FULL_CIRCLE as f32),
                                              360.0*(figure.beta as f32) / (FULL_CIRCLE as f32));
                figure_polygon_idx = embedding.add_figure(figure_polygon);
                unique_figures_polygons.insert(figure_key, figure_polygon_idx);
            } else {
                figure_polygon_idx = *unique_figures_polygons.get(&figure_key).unwrap();
            }

            let graph_node_idx = figure_idx_to_node_idx[figure_idx];
            embedding.set_node_info(graph_node_idx, figure.center, figure_polygon_idx);
        }

        // add all edges

        let mut node_neighbours: Vec<HashSet<NodeIndex>> = vec![HashSet::new(); sand_graph.nodes.len()];
        let mut sides_count: Vec<usize> =  vec![0; sand_graph.nodes.len()];

        for (pos, node_figures) in &self.vertices_info {
            if ! node_figures.is_complete() {
                continue;
            }

            let figures_count = node_figures.figures.len();

            for i in 0..figures_count {
                let figure_node_info_1 = &node_figures.figures[i % figures_count] ;
                let figure_node_info_2 = &node_figures.figures[(i + 1) % figures_count];

                let node_1_idx = figure_idx_to_node_idx[figure_node_info_1.figure_idx];
                let node_2_idx = figure_idx_to_node_idx[figure_node_info_2.figure_idx];

                sides_count[node_1_idx] = figure_node_info_1.sides_count;

                node_neighbours[node_1_idx].insert(node_2_idx);
                node_neighbours[node_2_idx].insert(node_1_idx);
            }
        }

        for node_idx in sand_graph.non_sink_nodes() {
            if sides_count[node_idx] == 0 {
                continue;
            }

            for neighbour_idx in &node_neighbours[node_idx] {
                sand_graph.add_edge(node_idx, *neighbour_idx, 1);
            }
            if node_neighbours[node_idx].len() < sides_count[node_idx] {
                sand_graph.add_edge(node_idx, SandGraph::SINK_NODE,
                                    (sides_count[node_idx] - node_neighbours[node_idx].len()) as i32);
            }
        }


        SandPileModel { graph: sand_graph, embedding}
    }
}