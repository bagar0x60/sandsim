pub mod camera;

extern crate rand;

use piston::input::{GenericEvent, RenderArgs, Button};
use graphics::{Context, Graphics};
use model::SandPileModel;
use controller::SandPileController;
use graphics::math;
use model::region::Cuboid;
use self::camera::CameraController;
use piston_window::{PistonWindow, OpenGLWindow};
use camera_controllers::model_view_projection;
use gfx;
use gfx::{Resources, Slice, PipelineState};
use gfx::handle::Buffer;
use gfx_device_gl::Resources as GfxResources;

gfx_vertex_struct!( Vertex {
    a_pos: [f32; 3] = "a_pos",
});

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    u_model_view_proj: gfx::Global<[[f32; 4]; 4]> = "u_model_view_proj",
    a_color: gfx::Global<[f32; 4]> = "a_color",
    out_color: gfx::RenderTarget<gfx::format::Srgba8> = "o_color",
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
});

impl Vertex {
    fn new(a_pos: [f32; 3]) -> Vertex {
        Vertex { a_pos }
    }
}


/*
pub struct SandPileView {
    camera: Camera,
    // Why the hell should i store this value?
    // Why can't i call something like window.get_cursor()?
    current_cursor_position: math::Vec2d,
    left_click_position: math::Vec2d,
    left_mouse_pressed: bool,
}
*/

pub struct SandPileView<C: CameraController> {
    figures:  Vec<(Buffer<GfxResources, Vertex>, Slice<GfxResources>)>,
    pso: PipelineState<GfxResources, pipe::Meta>,
    camera: C,
}

use opengl_graphics::{GLSL, OpenGL};
use shader_version::Shaders;
use piston::window::Window;
use gfx::Factory;
use gfx::traits::FactoryExt;

impl<C: CameraController> SandPileView<C> {
    pub fn new<F: FactoryExt<GfxResources>>(factory: &mut F,
               model: &SandPileModel,
               opengl: OpenGL,
               camera: C) -> SandPileView<C> {

        let mut figures: Vec<(Buffer<GfxResources, Vertex>, Slice<GfxResources>)> = Vec::new();
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
            let figure = factory
                .create_vertex_buffer_with_slice(&vertex_data, &index_data[..]);
            figures.push(figure);
        }

        let glsl = opengl.to_glsl();
        let pso = factory.create_pipeline_simple(
            Shaders::new()
                .set(GLSL::V1_50, include_str!("../../shaders/main_150.glslv"))
                .get(glsl).unwrap().as_bytes(),
            Shaders::new()
                .set(GLSL::V1_50, include_str!("../../shaders/main_150.glslf"))
                .get(glsl).unwrap().as_bytes(),
            pipe::new()
        ).unwrap();

        SandPileView { camera, figures, pso }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, controller: &mut SandPileController) {
        self.camera.event(e);

        if let Some(_) = e.update_args() {
            controller.topple(1000000);
        }
    }


    pub fn draw<W: Window>(&self, window: &mut PistonWindow<W>, args: RenderArgs, sandpile_model: &SandPileModel) {
        use camera_controllers::CameraPerspective;
        use vecmath;

        window.encoder.clear(&window.output_color, [1.0, 1.0, 1.0, 1.0]);
        window.encoder.clear_depth(&window.output_stencil, 1.0);

        let draw_size = window.window.draw_size();

        let model = vecmath::mat4_id();
        let mut projection = CameraPerspective {
            fov: 90.0, near_clip: 0.1, far_clip: 1000.0,
            aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32)
        }.projection();

        let (vbuf, slice) = &self.figures[0];


        let mut data = pipe::Data {
            vbuf: vbuf.clone(),
            u_model_view_proj: [[0.0; 4]; 4],
            a_color: [0.3, 0.3, 0.3, 1.0],
            out_color: window.output_color.clone(),
            out_depth: window.output_stencil.clone(),
        };

        data.u_model_view_proj = model_view_projection(
            model,
            self.camera.camera(args.ext_dt).orthogonal(),
            projection
        );
        window.encoder.draw(slice, &self.pso, &data);
    }

}


/*
impl Camera {
    pub fn new() -> Camera {
        Camera {zoom: 956.0 / 1000.0, center: [0.0; 2], temp_shift: [0.0; 2]}
    }

    fn zoom(&mut self, coefficient: f64) {
        self.zoom += 0.1*coefficient*(self.zoom + 1.0);
    }

    fn set_temp_shift(&mut self, shift: math::Vec2d) {
        self.temp_shift = shift;
    }

    fn store_shift(&mut self) {
        self.center = self.get_center();
        self.temp_shift = [0.0; 2];
    }

    fn get_center(&self) -> math::Vec2d {
        math::add(self.center, self.temp_shift)
    }
}
*/



/*
impl SandPileView {
    pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    pub const BLUE3: [f32; 4] = [0.0, 0.0, 0.8, 1.0];
    pub const BLUE2: [f32; 4] = [0.2, 0.2, 1.0, 1.0];
    pub const BLUE1: [f32; 4] = [0.6, 0.6, 1.0, 1.0];
    pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    pub fn new(camera: Camera) -> SandPileView {
        SandPileView {
            camera,
            current_cursor_position: [0.0, 0.0],
            left_click_position: [0.0, 0.0],
            left_mouse_pressed: false
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, controller: &mut SandPileController) {
        if let Some(_) = e.update_args() {
            controller.topple(1000000);
        }

        if let Some([_, vertical_scroll]) = e.mouse_scroll_args() {
            self.camera.zoom(vertical_scroll);
        }

        if let Some(pos) = e.mouse_cursor_args() {
            self.current_cursor_position = pos;
            if self.left_mouse_pressed {
                let shift = math::sub(self.left_click_position, self.current_cursor_position);
                let shift = math::mul_scalar(shift, 1.0 / self.camera.zoom);
                self.camera.set_temp_shift(shift);
            }
        }

        if let Some(Button::Mouse(Left)) = e.press_args() {
            self.left_mouse_pressed = true;
            self.left_click_position = self.current_cursor_position.clone();
        }

        if let Some(Button::Mouse(Left)) = e.release_args() {
            self.left_mouse_pressed = false;
            self.camera.store_shift();
        }
    }

    pub fn draw<G: Graphics>(&self, args: RenderArgs, context: &Context, gl: &mut G, model: &SandPileModel) {
        use graphics::*;



        let [x_center, y_center] = self.camera.get_center();
        let context = context.zoom(self.camera.zoom).trans(-x_center, -y_center);

        let square = rectangle::square(0.0, 0.0, 1.0);
        let radius = if self.camera.zoom < 50.0 {0.0} else {0.01};
        let border = rectangle::Border {color: Self::BLACK, radius: radius};

        let white = Rectangle::new(Self::WHITE).border(border);
        let blue1 = Rectangle::new(Self::BLUE1).border(border);
        let blue2 = Rectangle::new(Self::BLUE2).border(border);
        let blue3 = Rectangle::new(Self::BLUE3).border(border);
        let black = Rectangle::new(Self::BLACK).border(border);

        for node_idx in model.graph.non_sink_nodes() {
            let degree = model.graph.nodes[node_idx].degree;
            let sand_count = model.graph.nodes[node_idx].sand.get();

            if degree - sand_count == 1 {
                continue;
            }

            let rectangle = match degree - sand_count {
                _ if degree - sand_count > 3  => white,
                3 => blue1,
                2 => blue2,
                1 => blue3,
                _ => black
            };

            let coords = model.embedding.node_to_coordinates(node_idx);
            let x = coords[0];
            let y = coords[1];

            let transform = context.trans(x, y).transform;

            // let color = rectangle.color;
            // Ellipse::new(color).draw(square, &context.draw_state, transform, gl);
            rectangle.draw(square, &context.draw_state, transform, gl);
        }
    }
}
*/