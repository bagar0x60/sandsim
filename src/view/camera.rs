pub use camera_controllers::*;
use piston::input::GenericEvent;

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
