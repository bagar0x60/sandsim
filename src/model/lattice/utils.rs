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
    pub vertices: Vec<(math::Vec3d<f32>, usize)>,
}

#[derive(Debug)]
pub(super) struct VertexFigures {
    pub figures: Vec<FigureVertexInfo>,
    check_for_new_figures: bool,
}

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::TrySendError::Full;

#[derive(Debug)]
pub(super) struct UniformTiling {
    pub side_size: f32,
    pub figures: DataOnPlane<FigureGeometricInfo>,
    pub vertices_info: DataOnPlane<VertexFigures>,
}

pub(super) struct Constructor {
    pub tiling: UniformTiling
}

#[derive(Debug)]
pub struct DataOnPlane<T> {
    pub data: Vec<(math::Vec3d<f32>, T)>,
    plane: HashMap<(i64, i64), Vec<usize>>,
}


// --------------- implementations



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
        let beta = FULL_CIRCLE / 2 - alpha;
        let r = side_size / (2.0 * (to_radians(alpha)  / 2.0).cos() );
        let gamma = rotate + alpha / 2;
        let gamma_rad = to_radians(gamma);
        let center = vecmath::vec3_add([r*gamma_rad.cos(), r*gamma_rad.sin(), 0.0], position);
        let vertices = Self::compute_vertices(rotate, alpha, beta, sides_count, center, r);

        FigureGeometricInfo { center, rotate, sides_count, side_size, r, alpha, beta, vertices }
    }

    fn compute_vertices(rotate: usize,
                        alpha: usize,
                        beta: usize,
                        sides_count: usize,
                        center: math::Vec3d<f32>,
                        r: f32
    ) -> Vec<(math::Vec3d<f32>, usize)> {
        let mut result: Vec<(math::Vec3d<f32>, usize)> = Vec::new();

        let mut gamma = (rotate + alpha / 2 + FULL_CIRCLE / 2)  % FULL_CIRCLE;
        let mut rotate = rotate;
        for i in 0..sides_count {
            let gamma_rad = to_radians(gamma);
            let position =  vecmath::vec3_add(
                center,
                [r*gamma_rad.cos(), r*gamma_rad.sin(), 0.0]
            );

            result.push((position, rotate));

            gamma = (gamma + beta) % FULL_CIRCLE;
            rotate = (rotate + beta) % FULL_CIRCLE;
        }

        result
    }
}


impl UniformTiling {
    pub fn new(side_size: f32) -> Self {
        let vertices_info: DataOnPlane<VertexFigures> = DataOnPlane::new();
        let figures: DataOnPlane<FigureGeometricInfo> = DataOnPlane::new();

        UniformTiling { vertices_info, figures, side_size }
    }

    pub fn add_figure(&mut self, figure: FigureGeometricInfo) -> usize {
        let figure_idx = self.figures.data.len();

        for (pos1, rotate) in &figure.vertices {
            let figure_node_info = FigureVertexInfo { angle: *rotate, sides_count: figure.sides_count, figure_idx};
            let mut is_vertex_exist = false;

            if let Some(vertex_idx) = self.vertices_info.get_point_by_coords(*pos1) {
                let (_, vertex_figures) = &mut self.vertices_info.data[vertex_idx];
                vertex_figures.add(figure_node_info);
            } else {
                let mut vertex_figures = VertexFigures::new();
                vertex_figures.add(figure_node_info);
                self.vertices_info.add(*pos1, vertex_figures);
            }
        }

        self.figures.add(figure.center, figure);

        figure_idx
    }

    fn is_there_figure_in_point(&self, point: math::Vec3d<f32>) -> bool {
        if let Some(_) = self.figures.get_point_by_coords(point) {
            true
        } else {
            false
        }
    }

    pub fn build(&self) -> SandPileModel {
        let mut sand_graph = SandGraph::new();
        let mut embedding = EmbeddingToR3::new();

        // add nodes to graph
        let mut figure_idx_to_node_idx: Vec<NodeIndex> = Vec::new();

        for _ in &self.figures.data {
            let node_idx = sand_graph.add_node();
            figure_idx_to_node_idx.push(node_idx);
        }

        // add nodes coords to embedding
        // add figures to embedding
        let mut unique_figures_polygons: HashMap<(usize, usize), usize> = HashMap::new();

        for (figure_idx, (_, figure)) in self.figures.data.iter().enumerate() {
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


        for (_, node_figures) in &self.vertices_info.data {
            if ! node_figures.is_complete() {
                continue;
            }

            let figures_count = node_figures.figures.len();

            for i in 0..figures_count {
                let figure_node_info_1 = &node_figures.figures[i % figures_count] ;
                let figure_node_info_2 = &node_figures.figures[(i + 1) % figures_count];

                let node_1_idx = figure_idx_to_node_idx[figure_node_info_1.figure_idx];
                let node_2_idx = figure_idx_to_node_idx[figure_node_info_2.figure_idx];

                node_neighbours[node_1_idx].insert(node_2_idx);
                node_neighbours[node_2_idx].insert(node_1_idx);
            }
        }

        for node_idx in sand_graph.non_sink_nodes() {
            let sides_count = self.figures.data[node_idx - 1].1.sides_count;

            for neighbour_idx in &node_neighbours[node_idx] {
                sand_graph.add_edge(node_idx, *neighbour_idx, 1);
            }
            if node_neighbours[node_idx].len() < sides_count {
                sand_graph.add_edge(node_idx, SandGraph::SINK_NODE,
                                    (sides_count - node_neighbours[node_idx].len()) as i32);
            }
        }


        SandPileModel { graph: sand_graph, embedding}
    }
}


impl Constructor {
    pub fn new(origin: math::Vec3d<f32>, rotate_in_degrees: usize, side_size: f32, origin_figure_sides_count: usize) -> Self {
        let mut tiling = UniformTiling::new(side_size);
        let rotate = FULL_CIRCLE * rotate_in_degrees / 360;
        let origin_figure =
            FigureGeometricInfo::new(origin, rotate, side_size, origin_figure_sides_count);
        tiling.add_figure(origin_figure);

        Constructor { tiling }
    }

    pub fn add(&mut self, figure_idx: usize, side_idx: usize, sides_count: usize) -> usize {
        let other_figure_sides_count = self.tiling.figures.data[figure_idx].1.sides_count;
        assert!(side_idx < other_figure_sides_count);
        let other_figure_alpha = self.tiling.figures.data[figure_idx].1.alpha;
        let vertex_idx = (side_idx + 1) % other_figure_sides_count;

        let (position, other_rotate) = self.tiling.figures.data[figure_idx].1.vertices[vertex_idx];
        let rotate = (other_rotate + other_figure_alpha) % FULL_CIRCLE;

        let new_figure = FigureGeometricInfo::new(position, rotate, self.tiling.side_size, sides_count);
        self.tiling.add_figure(new_figure)
    }

    pub fn get_vector(&self, figure_idx_1: usize, figure_idx_2: usize) -> math::Vec3d<f32> {
        let v1 = self.tiling.figures.data[figure_idx_1].1.center;
        let v2 = self.tiling.figures.data[figure_idx_2].1.center;

        vecmath::vec3_sub(v2, v1)
    }
}

fn get_vectors_limits(origin: math::Vec3d<f32>, v1: math::Vec3d<f32>, v2: math::Vec3d<f32>, cuboid_hull: &Cuboid) -> ((f32, f32), (f32, f32)) {
    let [x0, y0, _] = origin;
    let ([x1, y1, _], [x2, y2, _]) = (v1, v2);


    let a = [
        [x1, x2],
        [y1, y2]
    ];

    let a_det = a[0][0]*a[1][1] - a[0][1]*a[1][0];
    assert!(a_det.abs() > 0.0001, "Determinant of two vectors is too small, they are probably collinear");

    let a_inv = [
        [a[1][1] / a_det, -a[0][1] / a_det],
        [-a[1][0] / a_det, a[0][0] / a_det]
    ];

    let [x_size, y_size, _] = *cuboid_hull;

    let  (mut v1_min, mut v1_max) = (std::f32::MAX, std::f32::MIN);
    let (mut v2_min, mut v2_max) = (std::f32::MAX, std::f32::MIN);

    let cuboid_vertices = [(0.0, 0.0), (0.0, y_size), (x_size, 0.0), (x_size, y_size)];
    for (x, y) in cuboid_vertices.iter() {
        let x_new = *x*a_inv[0][0] + *y*a_inv[0][1];
        let y_new = *x*a_inv[1][0] + *y*a_inv[1][1];

        v1_min = x_new.min(v1_min);
        v1_max = x_new.max(v1_max);

        v2_min = y_new.min(v2_min);
        v2_max = y_new.max(v2_max);
    }

    let x0_new = x0*a_inv[0][0] + y0*a_inv[0][1];
    let y0_new = x0*a_inv[1][0] + y0*a_inv[1][1];
    println!("{} {}", x0_new, y0_new);
    println!("{} {}, {} {}", v1_min, v1_max, v2_min, v2_max);


    ((v1_min - x0_new, v1_max - x0_new), (v2_min - y0_new, v2_max - y0_new))
}

pub(super) fn continue_tiling_by_translation(uniform_tiling: &mut UniformTiling,
                                      translation_vectors: (math::Vec3d<f32>, math::Vec3d<f32>),
                                      cuboid_hull: &Cuboid) {
    let (v1, v2) = translation_vectors;
    println!("v1 = {:?} v2 = {:?}", v1, v2);
    let origin = uniform_tiling.figures.data[0].1.center;

    let ((v1_min, v1_max), (v2_min, v2_max)) = get_vectors_limits(origin, v1, v2, cuboid_hull);

    let base_figures_count = uniform_tiling.figures.data.len();

    for i in (v1_min.floor() as i32)..=(v1_max.ceil() as i32) {
        for j in (v2_min.floor() as i32)..=(v2_max.ceil() as i32) {
            let translation = vecmath::vec3_add(
                vecmath::vec3_scale(v1, i as f32),
                vecmath::vec3_scale(v2, j as f32),
            );

            let new_origin = vecmath::vec3_add(origin, translation);
            if 0.0 <= new_origin[0] && new_origin[0] <= cuboid_hull[0] &&
                0.0 <= new_origin[1] && new_origin[1] <= cuboid_hull[1] {

                for figure_idx in 0..base_figures_count {
                    let center = vecmath::vec3_add(uniform_tiling.figures.data[figure_idx].1.center, translation);

                    if ! uniform_tiling.is_there_figure_in_point(center) {
                        let (position_old, rotate) = uniform_tiling.figures.data[figure_idx].1.vertices[0];
                        let position =  vecmath::vec3_add(position_old, translation);
                        let side_size = uniform_tiling.side_size;
                        let sides_count = uniform_tiling.figures.data[figure_idx].1.sides_count;
                        let new_figure = FigureGeometricInfo::new(position, rotate,side_size, sides_count);

                        uniform_tiling.add_figure(new_figure);
                    }
                }
            }
        }
    }
}


impl<T> DataOnPlane<T> {
    pub fn new() -> Self {
        DataOnPlane { data: Vec::new(), plane: HashMap::new() }
    }

    pub fn get_point_codes(position: Vec3d<f32>) -> [(i64, i64); 4] {
        let [x, y, _] = position;

        let mut result = [(0, 0); 4];
        let (x1, y1) = (x.abs().floor() as i64, y.abs().floor() as i64);
        let mut xs: Vec<i64> = vec![x1];
        let mut ys: Vec<i64> = vec![y1];
        if x.fract().abs() > 0.5 {
            xs.push(x1 + 1);
        } else {
            xs.push(x1 - 1);
        }
        if y.fract().abs() > 0.5 {
            ys.push(y1 + 1);
        } else {
            ys.push(y1 - 1);
        }

        [(xs[0], ys[0]), (xs[1], ys[0]), (xs[0], ys[1]), (xs[1], ys[1])]
    }

    pub fn add(&mut self, position: Vec3d<f32>, d: T) {
        let d_idx = self.data.len();
        self.data.push((position, d));

        for code in Self::get_point_codes(position).iter() {
            if self.plane.contains_key(code) {
                self.plane.get_mut(code).unwrap().push(d_idx);
            } else {
                self.plane.insert(*code, vec![d_idx]);
            }
        }
    }

    fn get_point_by_coords(&self, coords: math::Vec3d<f32>) -> Option<usize> {
        for code in Self::get_point_codes(coords).iter() {
            if let Some(points) = self.plane.get(code) {
                for data_idx in points {
                    let (pos, _) = &self.data[*data_idx];
                    if pretty_close(*pos, coords) {
                        return Some(*data_idx)
                    }
                }
            }
        }
        None
    }
}


