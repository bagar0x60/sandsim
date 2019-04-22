use piston::input::{GenericEvent, RenderArgs, Button};
use graphics::{Context, Graphics};
use model::SandPileModel;
use controller::SandPileController;
use vecmath::{vec3_add, vec3_sub, vec2_sub};
use graphics::math;

pub struct Camera {
    zoom: f64,
    center: math::Vec2d,
    temp_shift: math::Vec2d,
}

pub struct SandPileView {
    camera: Camera,
    // Why the hell should i store this value?
    // Why can't i call something like window.get_cursor()?
    current_cursor_position: math::Vec2d,
    left_click_position: math::Vec2d,
    left_mouse_pressed: bool,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {zoom: 1.0, center: [0.0; 2], temp_shift: [0.0; 2]}
    }

    fn zoom(&mut self, coefficient: f64) {
        self.zoom += coefficient;
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

impl SandPileView {
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    const BLUE3: [f32; 4] = [0.0, 0.0, 0.8, 1.0];
    const BLUE2: [f32; 4] = [0.2, 0.2, 1.0, 1.0];
    const BLUE1: [f32; 4] = [0.6, 0.6, 1.0, 1.0];
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    pub fn new() -> SandPileView {
        SandPileView {
            camera: Camera::new(),
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
                self.camera.set_temp_shift(math::sub(self.left_click_position, self.current_cursor_position))
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




        // some others shit
        //self.camera.zoom += 0.01;
        // self.camera.center = [10.0, 10.0, 0.0];
    }

    pub fn draw<G: Graphics>(&self, args: RenderArgs, context: &Context, gl: &mut G, model: &SandPileModel) {
        use graphics::*;

        let [x_center, y_center] = self.camera.get_center();
        let context = context.zoom(self.camera.zoom).trans(-x_center, -y_center);

        let square = rectangle::square(0.0, 0.0, 1.0);

        for node_idx in model.graph.non_sink_nodes() {
            let degree = model.graph.nodes[node_idx].degree;
            let sand_count = model.graph.nodes[node_idx].sand.get();
            let colour = match degree - sand_count {
                _ if degree - sand_count > 3  => Self::WHITE,
                3 => Self::BLUE1,
                2 => Self::BLUE2,
                1 => Self::BLUE3,
                _ => Self::BLACK
            };

            let coords = model.embedding.node_to_coordinates(node_idx);
            let x = coords[0];
            let y = coords[1];

            let transform = context.trans(x, y).transform;

            rectangle(colour, square, transform, gl);
        }
    }
}