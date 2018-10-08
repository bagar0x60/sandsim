extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };


#[derive(Clone)]
pub struct Field {
    interior: Vec<Vec<bool>>,
    size: (usize, usize),
    neighbours: Vec<Vec<Vec<(usize, usize)>>>,
}

impl Field {
    fn compute_neighbours(interior: &Vec<Vec<bool>>, size: (usize, usize)) -> Vec<Vec<Vec<(usize, usize)>>> {
        let mut res = vec![vec![vec![]; size.1]; size.0];
        for y in 0..size.0 {
            for x in 0..size.1 {
                let (x, y) = (x as i32, y as i32);
                let mut neighbours: Vec<(usize, usize)> = Vec::new();
                for (xn, yn) in &[(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                    if  0 <= *xn && *xn < size.1 as i32 &&
                        0 <= *yn && *yn < size.0 as i32 &&
                        interior[*yn as usize][*xn as usize] {
                        neighbours.push((*xn as usize, *yn as usize));
                    }
                }
                res[y as usize][x as usize] = neighbours;
            }
        }
        res
    }

    fn square(edge_size: usize) -> Field {
        let interior = vec![vec![true; edge_size]; edge_size];
        let size = (edge_size, edge_size);
        let neighbours = Field::compute_neighbours(&interior, size);
        Field{interior, size, neighbours}
    }

    fn circle(radius: usize) -> Field {
        let size = (2*radius + 1, 2*radius + 1);

        let mut interior = vec![vec![false; 2*radius + 1]; 2*radius + 1];

        let radius = radius as i32;
        let (x_center, y_center) = (radius, radius);

        for x in 0..(2*radius + 1) {
            for y in 0..(2*radius + 1) {
                interior[y as usize][x as usize] =
                    (x - x_center).pow(2) + (y - y_center).pow(2) <= radius.pow(2);
            }
        }
        let neighbours = Field::compute_neighbours(&interior, size);
        Field{interior, size, neighbours}
    }

    fn is_inside(&self, (x, y): (usize, usize)) -> bool {
        x < self.size.0 && y < self.size.1 && self.interior[y][x]
    }
}





pub struct SandPile {
    field: Field,
    sand: Vec<Vec<u32>>,
    stack: Vec<(usize, usize)>,
}

impl SandPile {
    fn new(field: Field, sand_level: u32) -> SandPile {
        let (x_max, y_max) = field.size;
        let sand = vec![vec![sand_level; x_max]; y_max];
        let mut stack: Vec<(usize, usize)> = Vec::new();
        if sand_level > 3 {
            for x in 0..x_max {
                for y in 0..y_max {
                    if field.is_inside((x, y)) {
                        stack.push((x, y));
                    }
                }
            }
        }

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

                    for (x_neighbor, y_neighbor) in self.field.neighbours[y][x].iter() {
                        self.sand[*y_neighbor][*x_neighbor] += 1;
                        if self.sand[*y_neighbor][*x_neighbor] > 3 {
                            new_stack.push((*x_neighbor, *y_neighbor))
                        }
                    }
                }
            }

            self.stack = new_stack;
        }
    }

    fn relax(&mut self) {
        while ! self.stack.is_empty() {
            self.topple(1);
        }
    }

    fn is_relaxed(&self) -> bool {
        self.stack.is_empty()
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


use std::ops;
impl ops::Add<SandPile> for SandPile {
    type Output = SandPile;

    fn add(self, other: SandPile) -> SandPile {
        if self.field.size != other.field.size {
            panic!("Sum of SandPiles of different field sizes isn't allowed");
        }
        let mut res = SandPile::new(self.field.clone(), 0);
        for x in 0..self.field.size.0 {
            for y in 0..self.field.size.1 {
                res.add_grains((x, y), self.sand[y][x] + other.sand[y][x]);
            }
        }
        res
    }
}

impl ops::Sub for SandPile {
    type Output = SandPile;

    fn sub(self, other: SandPile) -> SandPile {
        if self.field.size != other.field.size {
            panic!("Sum of SandPiles of different field sizes isn't allowed");
        }
        let mut res = SandPile::new(self.field.clone(), 0);
        for x in 0..self.field.size.0 {
            for y in 0..self.field.size.1 {
                res.add_grains((x, y), self.sand[y][x] - other.sand[y][x]);
            }
        }
        res
    }
}





pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
}

impl App {
    fn render(&mut self, args: &RenderArgs, sand: &Vec<Vec<u32>>) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const BLUE:  [f32; 4] = [0.0, 0.0, 1.0, 1.0];

        const YELLOW:[f32; 4] = [1.0, 1.0, 0.0, 1.0];


        let square_size: f64 = ((args.height as f64) / (sand.len() as f64))
                                    .min((args.width as f64) / (sand[0].len() as f64))
                                    .floor();
        let square = rectangle::square(0.0, 0.0, square_size);


        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(WHITE, gl);
            for (y, line) in sand.iter().enumerate() {
                for (x, cell) in line.iter().enumerate() {
                    let colour = match *cell  {
                        0 => GREEN,
                        1 => YELLOW,
                        2 => BLUE,
                        3 => RED,
                        _ => BLACK
                    };
                    let transform = c.transform.trans(square_size*(x as f64), square_size*(y as f64));

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
    };

    let mut sand_pile1 = SandPile::new(Field::square(200), 6);
    let mut sand_pile = SandPile::new(Field::square(200), 6);

    sand_pile1.relax();
    sand_pile = sand_pile - sand_pile1;

    // sand_pile.add_grains((50, 50), 1_000_000);
    // sand_pile.add_grains((75, 75), 2);



    let mut events = Events::new(EventSettings::new());
    loop {
        if



        if let Some(e) = events.next(&mut window){
            if let Some(r) = e.render_args() {
                app.render(&r, &sand_pile.sand);
            }
        }

        sand_pile.topple(1);

    }
}