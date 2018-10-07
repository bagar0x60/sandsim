extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };



pub struct Field {
    interior: Vec<Vec<bool>>,
    size: (usize, usize),
}

impl Field {
    fn square(edge_size: usize) -> Field {
        let interior = vec![vec![true; edge_size]; edge_size];
        let size = (edge_size, edge_size);
        return Field{interior, size};
    }

    fn is_inside(&self, (x, y): (usize, usize)) -> bool {
        x < self.size.0 && y < self.size.1 && self.interior[y][x]
    }

    fn neighbours(&self, (x, y): (usize, usize)) -> Vec<(usize, usize)> {
        let (x, y) = (x as i32, y as i32);
        let mut res: Vec<(usize, usize)> = Vec::new();
        for (xn, yn) in &[(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if  0 <= *xn && 0 <= *yn && self.is_inside((*xn as usize, *yn as usize)) {
                res.push((*xn as usize, *yn as usize));
            }
        }
        res
    }
}





pub struct SandPile {
    field: Field,
    sand: Vec<Vec<u32>>,
    stack: Vec<(usize, usize)>,
}

impl SandPile {
    fn new(field: Field) -> SandPile {
        let (x_max, y_max) = field.size;
        let sand = vec![vec![3; x_max]; y_max];
        let stack: Vec<(usize, usize)> = Vec::new();

        return SandPile {field, sand, stack}
    }

    fn topple(&mut self, rounds: u32) {
        for _ in 0..rounds {
            if self.stack.is_empty() {
                return;
            }

            let mut new_stack: Vec<(usize, usize)> = Vec::new();
            while let Some((x, y)) = self.stack.pop() {
                if self.sand[y][x] > 3 {
                    self.sand[y][x] -= 4;

                    for (x_neighbor, y_neighbor) in self.field.neighbours((x, y)) {
                        self.sand[y_neighbor][x_neighbor] += 1;
                        if self.sand[y_neighbor][x_neighbor] > 3 {
                            new_stack.push((x_neighbor, y_neighbor))
                        }
                    }
                }
            }

            self.stack = new_stack;
        }
    }


    fn add_grains(&mut self, (x, y): (usize, usize), count: u32) {
        if self.field.is_inside((x, y)) {
            self.sand[y][x] += count;
            if self.sand[y][x] > 3 {
                self.stack.push((x, y));
            }
        }
    }

}


pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64   // Rotation for the square.
}

impl App {
    fn render(&mut self, args: &RenderArgs, sand: &Vec<Vec<u32>>) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const YELLOW:[f32; 4] = [1.0, 1.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 10.0);
        // let (x, y) = ((args.width / 2) as f64, (args.height / 2) as f64);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(WHITE, gl);
            for (y, line) in sand.iter().enumerate() {
                for (x, cell) in line.iter().enumerate() {
                    let colour = match *cell  {
                        0 => RED,
                        1 => YELLOW,
                        2 => GREEN,
                        3 => WHITE,
                        _ => BLACK
                    };
                    let transform = c.transform.trans(10.0*(x as f64), 10.0*(y as f64));

                    rectangle(colour, square, transform, gl);
                }
            }
        });
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "spinning-square",
            [200, 200]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        rotation: 0.0
    };

    let field = Field::square(200);
    let mut sand_pile = SandPile::new(field);
    sand_pile.add_grains((25, 25), 2);
    sand_pile.add_grains((75, 75), 2);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r, &sand_pile.sand);
            // println!("{:?}", r);
        }

        sand_pile.topple(1);

    }
}