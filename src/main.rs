#![allow(dead_code)]

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston_window;
extern crate sandsim;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{ GlGraphics, OpenGL };

use sandsim::{model::SandPileModel, view::SandPileView, controller::SandPileController};
use sandsim::model::{lattice::{SquareLattice, HexagonLattice, TriangleLattice, CubeLattice},
                     region::{Rectangle, Circle, Parallelepiped}};
use sandsim::view::camera::{ OrbitZoomCamera, OrbitZoomCameraSettings, FirstPerson, FirstPersonSettings, Camera2d };
use piston_window::{PistonWindow, OpenGLWindow, AdvancedWindow};
use graphics::math::Vec3d;
use graphics::line::Shape::Square;

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: PistonWindow<GlutinWindow> =
        WindowSettings::new("SandPile", [640, 480])
            .exit_on_esc(true)
            .samples(4)
            .opengl(opengl)
            .build()
            .unwrap();
    // window.set_capture_cursor(true);

    let mut events = Events::new(EventSettings::new()
        .ups(10)
        .max_fps(10));

    let lattice = CubeLattice::new();
    let region = Parallelepiped::new(100.0, 100.0, 100.0);

    let camera_2d = Camera2d::new();
    let first_person = FirstPerson::new(
        [0.5, 0.5, 4.0],
        FirstPersonSettings::keyboard_wasd()
    );
    let mut orbital = OrbitZoomCamera::new(
        [50.0, 50.0, 50.0],
        OrbitZoomCameraSettings::default().zoom_speed(10.0)
    );
    orbital.distance = 120.0;

    let camera = orbital;

    let model = SandPileModel::new(region, lattice);
    let mut controller = SandPileController::new(model);
    let mut view =
        SandPileView::new(  &mut window.factory, &controller.model, opengl, camera);


    controller.clear_sand();
    controller.max_stable();

    controller.add_sand([65.0, 20.0, 40.0], 1);
    controller.add_sand([33.0, 51.0, 29.0], 1);


    // controller.add_sand([150.0, 150.0, 0.0], 1);

    // controller.add_sand([400.0, 400.0, 0.0], 1);
    // controller.add_sand([350.0, 100.0, 0.0], 1);
    // controller.add_sand([100.0, 350.0, 0.0], 1);

    while let Some(e) = events.next(&mut window) {
        view.event(&e, &mut controller);

        if let Some(args) = e.render_args() {
            window.window.make_current();
            window.encoder.clear(&window.output_color, [0.9, 0.9, 0.9, 0.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);

            view.draw(&mut window, args, &controller.model);

            window.encoder.flush(&mut window.device);
        }
    }
}