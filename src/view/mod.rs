use piston::input::{GenericEvent, RenderArgs};
use graphics::{Context, Graphics};
use model::SandPileModel;
use controller::SandPileController;

pub struct Camera {
}

pub struct SandPileView {
    camera: Camera,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {}
    }
}

impl SandPileView {
    pub fn new() -> SandPileView {
        SandPileView {camera: Camera::new()}
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, controller: &mut SandPileController) {
        if let Some(_) = e.update_args() {
            controller.topple();
        }

        // some others shit
    }

    pub fn draw<G: Graphics>(&self, args: RenderArgs, c: &Context, gl: &mut G, model: &SandPileModel) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLUE3: [f32; 4] = [0.0, 0.0, 0.8, 1.0];
        const BLUE2: [f32; 4] = [0.2, 0.2, 1.0, 1.0];
        const BLUE1: [f32; 4] = [0.6, 0.6, 1.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let square_size: f64 = ((args.height as f64) / (200.0 as f64))
            .min((args.width as f64) / (200.0 as f64));

        let square = rectangle::square(0.0, 0.0, square_size);

        for node_idx in model.graph.non_sink_nodes() {
            let sand_count = &model.graph.nodes[node_idx].sand;
            let colour = match sand_count.get()  {
                0 => WHITE,
                1 => BLUE1,
                2 => BLUE2,
                3 => BLUE3,
                _ => BLACK
            };

            let coords = model.embedding.node_to_coordinates(node_idx);
            let x = coords[0];
            let y = coords[1];

            let transform = c.transform.trans(square_size*(x as f64), square_size*(y as f64));

            rectangle(colour, square, transform, gl);
        }
    }
}