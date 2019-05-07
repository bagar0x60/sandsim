mod regular_lattice;

pub use self::regular_lattice::{SquareLattice, TriangleLattice, HexagonLattice, CubeLattice};

use graphics::math;
use model::SandPileModel;
use super::region::Cuboid;
use super::sand_graph::{SandGraph, NodeIndex};
use super::embedding::{ EmbeddingToR3, Figure };
use std::collections::linked_list::LinkedList;
use vecmath;
use graphics::math::Vec3d;


pub trait Lattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel;
}

const FULL_CIRCLE: usize = 720;


#[derive(Clone, Copy, Debug)]
struct FigureNodeInfo {
    sides_count: usize,
    node_idx: NodeIndex,
    angle: usize,
}

#[derive(Debug)]
struct FigureGeometricInfo {
    alpha: usize,
    beta: usize,
    r: f32,
    center: math::Vec3d<f32>,
    rotate: usize,
    side_size: f32,
    sides_count: usize,
}

#[derive(Debug)]
struct NodeFigures {
    figures: Vec<FigureNodeInfo>,
    check_for_new_figures: bool,
}

#[derive(Debug)]
pub struct SemiRegularLattice {
    tiling_code: Vec<usize>,
}

use regex::Regex;
impl NodeFigures {
    fn new() -> Self {
        NodeFigures { figures: Vec::new(), check_for_new_figures: true }
    }

    fn add(&mut self, figure: FigureNodeInfo) {
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

    fn new_figures(&mut self, tiling_code: &Vec<usize>) -> Vec<(usize, usize)> {
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

impl SemiRegularLattice {
    pub fn new(tiling_code: Vec<usize>) -> Self {
        let angle: usize = tiling_code.iter().map(|n| FULL_CIRCLE / 2 - FULL_CIRCLE / n).sum();
        assert_eq!(angle % FULL_CIRCLE, 0,
                   "Incorrect tiling code {:?}. Figures don't sum up to {} degrees", tiling_code, FULL_CIRCLE);

        SemiRegularLattice { tiling_code }
    }
}

fn to_radians(angle: usize) -> f32 {
    (360.0 * (angle as f32) / (FULL_CIRCLE as f32)).to_radians()
}

impl FigureGeometricInfo {
    fn new(position: math::Vec3d<f32>, rotate: usize, side_size: f32, sides_count: usize) -> Self {
        let rotate = rotate % FULL_CIRCLE;
        let alpha = (FULL_CIRCLE / 2 - FULL_CIRCLE / sides_count) % FULL_CIRCLE;
        let r = side_size / (2.0 * (to_radians(alpha)  / 2.0).cos() );
        let gamma = rotate + alpha / 2;
        let gamma_rad = to_radians(gamma);
        let center = vecmath::vec3_add([r*gamma_rad.cos(), r*gamma_rad.sin(), 0.0], position);
        FigureGeometricInfo { center, rotate, sides_count, side_size, r, alpha, beta: FULL_CIRCLE / 2 - alpha }
    }

    fn vertices(&self) -> Vec<(math::Vec3d<f32>, usize)> {
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


fn add_figure(nodes_info: &mut Vec<(math::Vec3d<f32>, NodeFigures)>,
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
        let figure_node_info = FigureNodeInfo { angle: rotate, sides_count: figure.sides_count, node_idx: graph_node_idx};
        let mut is_vertex_exist = false;

        for (pos2, nodes_figures) in nodes_info.iter_mut() {
            if pretty_close(pos1, *pos2) {
                nodes_figures.add(figure_node_info);
                is_vertex_exist = true;
                break;
            }
        }

        if ! is_vertex_exist {
            let mut node_figures = NodeFigures::new();
            node_figures.add(figure_node_info);
            nodes_info.push((pos1, node_figures));
        }
    }
}

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::TrySendError::Full;

impl Lattice for SemiRegularLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        let side_size = 1.0_f32;

        let [x_size, y_size, _] = *cuboid_hull;

        let mut sand_graph = SandGraph::new();
        let mut embedding = EmbeddingToR3::new();

        let mut nodes_info: Vec<(math::Vec3d<f32>, NodeFigures)> = Vec::new();

        let mut unique_figures: HashMap<(usize, usize), usize> = HashMap::new();


        // add figures from self.tiling_code around [0.0, 0.0, 0.0]
        let mut rotate = 0;
        for sides_count in &self.tiling_code {
            let figure =
                FigureGeometricInfo::new([x_size / 2.0, y_size / 2.0, 0.0], rotate, side_size, *sides_count);

            rotate = (rotate + figure.alpha) % FULL_CIRCLE;
            add_figure(&mut nodes_info, figure, &mut sand_graph, &mut embedding, &mut unique_figures);
        }

        // println!("{:#?}", nodes_info);

        // add all other figures
        loop {
            let mut new_figures: Vec<FigureGeometricInfo> = Vec::new();

            for (pos, node_figures) in &mut nodes_info {
                let [x, y, _] = *pos;
                if 0.0 <= x && x < x_size && 0.0 <= y && y < y_size {
                    let new_figures_partial_info = node_figures.new_figures(&self.tiling_code);

                    if new_figures_partial_info.len() > 0 {
                        new_figures = new_figures_partial_info
                            .iter()
                            .map(|(angle, sides_count)| FigureGeometricInfo::new(*pos, *angle, side_size, *sides_count))
                            .collect();
                        break
                    }
                }
            }

            if new_figures.len() > 0 {
                for figure in new_figures {
                    add_figure(&mut nodes_info, figure, &mut sand_graph, &mut embedding, &mut unique_figures);
                }
             } else {
                break
            }
        }


        // println!("{:#?}", nodes_info);

        // add all edges

        let mut node_neighbours: Vec<HashSet<NodeIndex>> = vec![HashSet::new(); sand_graph.nodes.len()];
        let mut sides_count: Vec<usize> =  vec![0; sand_graph.nodes.len()];

        for (pos, node_figures) in &nodes_info {
            let figures_count = node_figures.figures.len();
            if figures_count < self.tiling_code.len() {
                continue;
            }

            // println!("{:#?}", node_figures);

            for i in 0..figures_count {
                let figure_node_info_1 = &node_figures.figures[i % figures_count] ;
                let figure_node_info_2 = &node_figures.figures[(i + 1) % figures_count];

                sides_count[figure_node_info_1.node_idx] = figure_node_info_1.sides_count;

                node_neighbours[figure_node_info_1.node_idx].insert(figure_node_info_2.node_idx);
                node_neighbours[figure_node_info_2.node_idx].insert(figure_node_info_1.node_idx);
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


















