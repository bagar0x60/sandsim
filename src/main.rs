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
use sandsim::model::{lattice::{SquareLattice, HexagonLattice, TriangleLattice, CubeLattice, SemiRegularLattice, KUniformLattice},
                     region::{Rectangle, Circle, Parallelepiped, Hexagon}};
use sandsim::view::camera::{ OrbitZoomCamera, OrbitZoomCameraSettings, FirstPerson, FirstPersonSettings, Camera2d };
use piston_window::{PistonWindow, OpenGLWindow, AdvancedWindow};
use graphics::math::Vec3d;
use graphics::line::Shape::Square;
use sandsim::model::region::Region;

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
        .ups(1000)
        .max_fps(60));

    // let lattice = SemiRegularLattice::new(vec![3, 3, 3, 4, 4]);
    // let lattice = CubeLattice::new();
    let lattice = KUniformLattice::new();


    // let region = Parallelepiped::new(200.0, 200.0, 100.0);
    let side = 100.0_f32;

    // let region = Hexagon::new(side / 3.0_f32.powf(0.5));
    //let region = Rectangle::new(side, side);
    let region = Circle::new(side / 2.0);
    // let region = Parallelepiped::new(side, side, side);

    let [cx, cy, cz] = region.cuboid_hull();

    let camera_2d = Camera2d::new();
    let first_person = FirstPerson::new(
        [0.5, 0.5, 4.0],
        FirstPersonSettings::keyboard_wasd()
    );
    let mut orbital = OrbitZoomCamera::new(
        [cx / 2.0, cy / 2.0, cz / 2.0],
        OrbitZoomCameraSettings::default().zoom_speed(10.0)
    );
    orbital.distance = side / 1.6;

    let camera = orbital;

    let mut model = SandPileModel::new(region, lattice);
    // model.transpose();
    // let f1 = |_, d| { 2*d };
    // let f2 = |s, d| { 2*d - s};

    let mut controller = SandPileController::new(model);

    // controller.set_pipeline(vec![&f1, &f2]);
    let mut view =
        SandPileView::new(  &mut window.factory, &controller.model, opengl, camera);


    controller.clear_sand();
    controller.max_stable();

    // println!("{:#?}", controller.model);


    controller.add_sand([0.5*side, 0.5*side, 0.0], 1);
    // controller.add_sand([0.4*side, 0.1*side, 0.0], 1);
    // controller.add_sand([0.7*side, 0.2*side, 0.0], 1);


    //controller.add_sand([0.8*side, 0.8*side, 0.0], 1);
    /*

    controller.add_sand([0.24*side, 0.89*side, 0.0], 1);
    controller.add_sand([0.4*side, 0.45*side, 0.0], 1);
    controller.add_sand([0.5*side, 0.4*side, 0.0], 1);
    controller.add_sand([0.5*side, 0.8*side, 0.0], 1);
    */

    // controller.add_sand([100.0, 100.0, 0.0], 1);
    // controller.add_sand([150.0, 150.0, 0.0], 1);
    // controller.add_sand([10.0, 70.0, 0.0], 1);

    // controller.add_sand([350.0, 100.0, 0.0], 1);
    // controller.add_sand([100.0, 350.0, 0.0], 1);

    // window.events.set_bench_mode(true);
    window.window.make_current();

    while let Some(e) = window.next() {
        view.event(&e, &mut controller);

        if let Some(args) = e.render_args() {
            /*
            if ! controller.is_relaxed() {
                println!("round");
                continue
            }
            */


            window.encoder.clear(&window.output_color, [0.9, 0.9, 0.9, 0.0]);
            window.encoder.clear_depth(&window.output_stencil, 1.0);

            view.draw_tiling(&mut window, args, &controller.model);

            window.encoder.flush(&mut window.device);
        }
    }
}