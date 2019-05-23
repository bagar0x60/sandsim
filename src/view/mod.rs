pub mod camera;

extern crate rand;

use piston::input::{GenericEvent, RenderArgs};
use model::SandPileModel;
use controller::SandPileController;
use self::camera::CameraController;
use piston_window::{PistonWindow};
use camera_controllers::model_view_projection;
use gfx;
use gfx::{Slice, PipelineState, Factory};
use gfx::handle::Buffer;
use gfx_device_gl::Resources as GfxResources;

gfx_vertex_struct!( Vertex {
    a_pos: [f32; 3] = "a_pos",
});

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    u_model_view_proj: gfx::Global<[[f32; 4]; 4]> = "u_model_view_proj",
    a_color: gfx::Global<[f32; 4]> = "a_color",
    out_color: gfx::BlendTarget<gfx::format::Srgba8> = ("o_color", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
});

impl Vertex {
    fn new(a_pos: [f32; 3]) -> Vertex {
        Vertex { a_pos }
    }
}

pub struct SandPileView<C: CameraController> {
    figures:  Vec<(Buffer<GfxResources, Vertex>, Slice<GfxResources>)>,
    borders:  Vec<(Buffer<GfxResources, Vertex>, Slice<GfxResources>)>,
    graph_lines: (Buffer<GfxResources, Vertex>, Slice<GfxResources>),
    main_pso: PipelineState<GfxResources, pipe::Meta>,
    line_pso: PipelineState<GfxResources, pipe::Meta>,
    view_projection: vecmath::Matrix4<f32>,
    camera: C,
}

use opengl_graphics::{GLSL, OpenGL};
use shader_version::Shaders;
use piston::window::Window;
use gfx::traits::FactoryExt;
use model::sand_graph::SandGraph;

impl<C: CameraController> SandPileView<C> {
    pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.1];
    pub const BLUE1: [f32; 4] = [0.6, 0.6, 1.0, 0.3];
    pub const BLUE2: [f32; 4] = [0.2, 0.2, 1.0, 0.5];
    pub const BLUE3: [f32; 4] = [0.0, 0.0, 0.8, 0.7];
    pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.9];
    pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 0.5];
    pub const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 0.5];
    pub const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];


    pub fn new<F: FactoryExt<GfxResources>>(
        factory: &mut F,
        model: &SandPileModel,
        opengl: OpenGL,
        camera: C) -> SandPileView<C> {

        let mut figures: Vec<(Buffer<GfxResources, Vertex>, Slice<GfxResources>)> = Vec::new();
        let mut borders: Vec<(Buffer<GfxResources, Vertex>, Slice<GfxResources>)> = Vec::new();
        for figure_data in &model.embedding.unique_figures {
            // let mut vertex_data: Vec<Vertex> = Vec::new();
            let vertex_data: Vec<Vertex> = figure_data.vertices
                .iter()
                .map(|&v| { Vertex::new(v) })
                .collect();
            let index_data: Vec<u32> = figure_data.indexes
                .iter()
                .map(|&x| {x as u32})
                .collect();
            let border_index_data: Vec<u32> = figure_data.border_indexes
                .iter()
                .map(|&x| {x as u32})
                .collect();

            let figure = factory
                .create_vertex_buffer_with_slice(&vertex_data, &index_data[..]);
            let border = factory
                .create_vertex_buffer_with_slice(&vertex_data, &border_index_data[..]);

            figures.push(figure);
            borders.push(border);
        }

        let main_pso = Self::build_main_pso(factory, opengl);
        let line_pso = Self::build_line_pso(factory);

        let graph_lines = Self::build_graph_lines(factory, &model);

        SandPileView { camera, figures, borders, graph_lines, main_pso, line_pso, view_projection: vecmath::mat4_id() }
    }

    pub fn get_veiw_projection(&self) -> vecmath::Matrix4<f32> {
        self.view_projection
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, controller: &mut SandPileController) {
        self.camera.event(e);

        /*
        if let Some(Button::Mouse(Left)) = e.press_args() {
            self.left_mouse_pressed = true;
            self.left_click_position = self.current_cursor_position.clone();
        }
        */

        if let Some(_) = e.update_args() {
            controller.update();
        }
    }

    fn compute_view_projection<W: Window>(&mut self, window: &mut PistonWindow<W>, args: RenderArgs) {
        use camera_controllers::CameraPerspective;

        let draw_size = window.window.draw_size();

        let projection = CameraPerspective {
            fov: 90.0,
            near_clip: 0.1,
            far_clip: 2000.0,
            aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32)
        }.projection();

        let view = self.camera.camera(args.ext_dt).orthogonal();

        self.view_projection = vecmath::col_mat4_mul(projection, view);
    }

    fn build_main_pso<F: FactoryExt<GfxResources>>(factory: &mut F, opengl: OpenGL) -> PipelineState<GfxResources, pipe::Meta> {
        let glsl = opengl.to_glsl();
        factory.create_pipeline_simple(
            Shaders::new()
                .set(GLSL::V1_50, include_str!("../../shaders/main_150.glslv"))
                .get(glsl).unwrap().as_bytes(),
            Shaders::new()
                .set(GLSL::V1_50, include_str!("../../shaders/main_150.glslf"))
                .get(glsl).unwrap().as_bytes(),
            pipe::new()
        ).unwrap()
    }

    fn build_line_pso<F: FactoryExt<GfxResources>>(factory: &mut F) -> PipelineState<GfxResources, pipe::Meta> {
        //set fillmode
        let mut fillmode = gfx::state::Rasterizer::new_fill();
        fillmode.method = gfx::state::RasterMethod::Line(1);
        //import shader
        let vs = include_bytes!("../../shaders/main_150.glslv");
        let fs = include_bytes!("../../shaders/main_150.glslf");

        let set = factory.create_shader_set(vs, fs).unwrap();
        // define pipeline
        factory.create_pipeline_state(
            &set,
            gfx::Primitive::LineList,
            fillmode,
            pipe::new()
        ).unwrap()
    }

    fn build_graph_lines<F: FactoryExt<GfxResources>>(factory: &mut F, sandpile_model: &SandPileModel) -> (Buffer<GfxResources, Vertex>, Slice<GfxResources>) {
        let mut vertex_data: Vec<Vertex> = Vec::new();
        let mut index_data: Vec<u32> = Vec::new();

        for node_idx in sandpile_model.graph.non_sink_nodes() {
            let (coords, _) = sandpile_model.embedding.get_node_info(node_idx);
            let origin_idx = vertex_data.len() as u32;
            vertex_data.push(Vertex::new(coords));

            for (_, neighbour_idx) in sandpile_model.graph.successors(node_idx) {
                if neighbour_idx == SandGraph::SINK_NODE {
                    continue;
                }

                let (neighbour_coords, _) = sandpile_model.embedding.get_node_info(neighbour_idx);
                index_data.push(vertex_data.len() as u32);
                index_data.push(origin_idx);

                vertex_data.push(Vertex::new(neighbour_coords));
            }
        }

        factory.create_vertex_buffer_with_slice(&vertex_data, &index_data[..])
    }

    pub fn draw_graph<W: Window>(&mut self, window: &mut PistonWindow<W>, args: RenderArgs, sandpile_model: &SandPileModel) {
        self.compute_view_projection(window, args);

        let out_color = window.output_color.clone();
        let out_depth = window.output_stencil.clone();

        let u_model_view_proj = self.view_projection;
        let a_color = Self::RED;

        let (vbuf, slice) = &self.graph_lines;

        let data = pipe::Data {
            vbuf: vbuf.clone(),
            u_model_view_proj,
            a_color,
            out_color: out_color.clone(),
            out_depth: out_depth.clone(),
        };

        window.encoder.draw(slice, &self.line_pso, &data);
    }

    pub fn draw_borders<W: Window>(&mut self, window: &mut PistonWindow<W>, args: RenderArgs, sandpile_model: &SandPileModel) {
        self.compute_view_projection(window, args);

        let out_color = window.output_color.clone();
        let out_depth = window.output_stencil.clone();

        for node_idx in sandpile_model.graph.non_sink_nodes() {
            let (coords, figure_idx) = sandpile_model.embedding.get_node_info(node_idx);
            let (vbuf, slice) = &self.borders[figure_idx];
            let [x, y, z] = coords;

            let model = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x, y, z, 1.0]
            ];
            let u_model_view_proj = vecmath::col_mat4_mul(self.view_projection, model);

            let sides_count = slice.get_prim_count(gfx::Primitive::TriangleList);
            let a_color = Self::BLACK;

            let data = pipe::Data {
                vbuf: vbuf.clone(),
                u_model_view_proj,
                a_color,
                out_color: out_color.clone(),
                out_depth: out_depth.clone(),
            };

            window.encoder.draw(slice, &self.line_pso, &data);
        }
    }

    pub fn draw_tiling<W: Window>(&mut self, window: &mut PistonWindow<W>, args: RenderArgs, sandpile_model: &SandPileModel) {
        self.compute_view_projection(window, args);

        let out_color = window.output_color.clone();
        let out_depth = window.output_stencil.clone();

        for node_idx in sandpile_model.graph.non_sink_nodes() {
            let (coords, figure_idx) = sandpile_model.embedding.get_node_info(node_idx);
            let (vbuf, slice) = &self.figures[figure_idx];
            let [x, y, z] = coords;

            let model = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x, y, z, 1.0]
            ];
            let u_model_view_proj = vecmath::col_mat4_mul(self.view_projection, model);

            let sides_count = slice.get_prim_count(gfx::Primitive::TriangleList);
            let a_color = match sides_count {
                _ if sides_count > 4 => Self::GREEN,
                4 =>  Self::YELLOW,
                3 => Self::BLUE3,
                2 => Self::GREEN,
                1 => Self::RED,
                _ => Self::WHITE
            };

            let (vbuf, slice) = &self.figures[figure_idx];

            let data = pipe::Data {
                vbuf: vbuf.clone(),
                u_model_view_proj,
                a_color,
                out_color: out_color.clone(),
                out_depth: out_depth.clone(),
            };

            window.encoder.draw(slice, &self.main_pso, &data);
            //break;
        }
    }

    pub fn draw<W: Window>(&mut self, window: &mut PistonWindow<W>, args: RenderArgs, sandpile_model: &SandPileModel) {
        self.compute_view_projection(window, args);

        let out_color = window.output_color.clone();
        let out_depth = window.output_stencil.clone();

        for node_idx in sandpile_model.graph.non_sink_nodes() {
            let degree = sandpile_model.graph.nodes[node_idx].degree;
            let sand_count = sandpile_model.graph.nodes[node_idx].sand.get();

            if degree - sand_count == 1 {
                continue;
            }


            let a_color = match degree - sand_count {
                _ if degree - sand_count > 3 => Self::BLACK,
                3 => Self::BLUE3,
                2 => Self::BLUE2,
                1 => Self::BLUE1,
                _ => Self::WHITE
            };

            let (coords, figure_idx) = sandpile_model.embedding.get_node_info(node_idx);
            let (vbuf, slice) = &self.figures[figure_idx];
            let [x, y, z] = coords;

            let model = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x, y, z, 1.0]
            ];
            let u_model_view_proj = vecmath::col_mat4_mul(self.view_projection, model);

            let (vbuf, slice) = &self.figures[figure_idx];

            let data = pipe::Data {
                vbuf: vbuf.clone(),
                u_model_view_proj,
                a_color,
                out_color: out_color.clone(),
                out_depth: out_depth.clone(),
            };

            window.encoder.draw(slice, &self.main_pso, &data);
        }
    }
}



