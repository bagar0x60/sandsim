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
use sandsim::model::{lattice::{SquareLattice, HexagonLattice, TriangleLattice}, region::{Rectangle, Circle}};
use sandsim::view::camera::{ OrbitZoomCamera, FirstPerson, FirstPersonSettings };
use graphics::line::Shape::Square;
use piston_window::{PistonWindow, OpenGLWindow, AdvancedWindow};

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: PistonWindow<GlutinWindow> =
        WindowSettings::new("SandPile", [640, 480])
            .exit_on_esc(true)
            .samples(4)
            .opengl(opengl)
            .build()
            .unwrap();
    window.set_capture_cursor(true);

    let ref mut factory = window.factory.clone();


    let mut events = Events::new(EventSettings::new());
    let mut gl = GlGraphics::new(opengl);


    let lattice = SquareLattice::new();
    let region = Rectangle::new(10.0, 10.0);
    let camera = FirstPerson::new(
        [0.5, 0.5, 4.0],
        FirstPersonSettings::keyboard_wasd()
    );
    let mut model = SandPileModel::new(region, lattice);
    let mut controller = SandPileController::new(model);
    let mut view =
        SandPileView::new(  &mut window.factory, &controller.model, opengl, camera);


    controller.clear_sand();
    controller.max_stable();
    // controller.add_sand([300.0, 300.0, 0.0], 1);
    // controller.add_sand([700.0, 200.0, 0.0], 1);
    // controller.add_sand([800.0, 800.0, 0.0], 1);

    while let Some(e) = events.next(&mut window) {
        view.event(&e, &mut controller);

        if let Some(args) = e.render_args() {
            window.window.make_current();
            view.draw(&mut window, args, &controller.model);
            window.encoder.flush(&mut window.device);
        }
    }
}