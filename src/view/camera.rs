pub use camera_controllers::*;
use piston::input::{ GenericEvent, Button };
use graphics::math;

pub trait CameraController {
    fn camera(&self, dt: f64) -> Camera<f32>;

    fn event<E: GenericEvent>(&mut self, e: &E);
}

impl CameraController for FirstPerson {
    fn camera(&self, dt: f64) -> Camera<f32> {
        self.camera(dt)
    }

    fn event<E: GenericEvent>(&mut self, e: &E) {
        self.event(e)
    }
}

impl CameraController for OrbitZoomCamera {
    fn camera(&self, dt: f64) -> Camera<f32> {
        self.camera(dt)
    }

    fn event<E: GenericEvent>(&mut self, e: &E) {
        self.event(e)
    }
}

pub struct Camera2d {
    zoom: f32,
    center: math::Vec2d<f32>,
    temp_shift: math::Vec2d<f32>,
    current_cursor_position: math::Vec2d<f32>,
    left_click_position: math::Vec2d<f32>,
    left_mouse_pressed: bool,
}

impl Camera2d {
    pub fn new() -> Camera2d {
        Camera2d {
            zoom: 956.0 / 1000.0,
            center: [0.0; 2],
            temp_shift: [0.0; 2],
            current_cursor_position: [0.0; 2],
            left_click_position:  [0.0; 2],
            left_mouse_pressed: false,
        }
    }

    fn zoom(&mut self, coefficient: f32) {
        self.zoom += coefficient;
    }

    fn set_temp_shift(&mut self, shift: math::Vec2d<f32>) {
        self.temp_shift = shift;
    }

    fn store_shift(&mut self) {
        self.center = self.get_center();
        self.temp_shift = [0.0; 2];
    }

    fn get_center(&self) -> math::Vec2d<f32> {
        math::add(self.center, self.temp_shift)
    }
}


impl CameraController for Camera2d {
    fn camera(&self, _dt: f64) -> Camera<f32> {
        let [x, y] = self.get_center();
        // println!("{:?}", [x, y, 1.0 / self.zoom]);

        Camera::new([x, y, 100.0 - self.zoom])
    }

    fn event<E: GenericEvent>(&mut self, e: &E) {
        if let Some([_, vertical_scroll]) = e.mouse_scroll_args() {
            self.zoom(vertical_scroll as f32);
        }

        if let Some([x, y]) = e.mouse_cursor_args() {
            self.current_cursor_position = [x as f32, -y as f32];
            if self.left_mouse_pressed {
                let shift = math::sub(self.left_click_position, self.current_cursor_position);
                // let shift = math::mul_scalar(shift, 1.0 / self.zoom);
                self.set_temp_shift(shift);
            }
        }

        if let Some(Button::Mouse(Left)) = e.press_args() {
            self.left_mouse_pressed = true;
            self.left_click_position = self.current_cursor_position.clone();
        }

        if let Some(Button::Mouse(Left)) = e.release_args() {
            self.left_mouse_pressed = false;
            self.store_shift();
        }
    }
}
