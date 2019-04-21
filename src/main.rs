#![allow(dead_code)]

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

extern crate sandsim;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

use sandsim::{model::SandPileModel, view::SandPileView, controller::SandPileController};
use sandsim::model::{lattice::SquareLattice, region::{Rectangle, Circle}};
use sandsim::model::lattice::HexagonLattice;


fn main() {
    let opengl = OpenGL::V3_2;
    let settings = WindowSettings::new("SandPile", [200, 200])
        .opengl(opengl)
        .exit_on_esc(true);
    let mut window: Window = settings.build()
        .expect("Could not create window");

    let mut events = Events::new(EventSettings::new());
    let mut gl = GlGraphics::new(opengl);


    let lattice = HexagonLattice::new();
    let region = Rectangle::new(1000.0, 1000.0);
    let mut model = SandPileModel::new(region, lattice);
    let mut controller = SandPileController::new(model);
    let mut view = SandPileView::new();


    controller.clear_sand();
    controller.max_stable();
    controller.add_sand([300.0, 300.0, 0.0], 1);
    // controller.add_sand([700.0, 200.0, 0.0], 1);

    while let Some(e) = events.next(&mut window) {
        view.event(&e, &mut controller);

        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                use graphics::{clear};

                clear([1.0; 4], g);
                view.draw(args, &c, g, &controller.model);
            });
        }
    }
}