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

#[derive(Clone)]
enum RelaxationStrategy {
    Stack,
    ArrayTraversing,
}



pub struct SandPile {
    field: Field,
    sand: Vec<Vec<u32>>,
    stack: Vec<(usize, usize)>,
    relaxation_strategy: RelaxationStrategy,
    is_relaxed: bool,
}

impl SandPile {
    fn new(field: Field, sand_level: u32) -> SandPile {
        let (x_max, y_max) = field.size;
        let sand = vec![vec![sand_level; x_max]; y_max];
        let mut stack: Vec<(usize, usize)> = Vec::new();
        let relaxation_strategy = RelaxationStrategy::ArrayTraversing;
        let is_relaxed = sand_level <= 3;

        SandPile {field, sand, stack, relaxation_strategy, is_relaxed}
    }

    fn build_stack(&mut self) {
        self.stack = Vec::new();
        for x in 0..self.field.size.0 {
            for y in 0..self.field.size.1 {
                if self.field.is_inside((x, y)) && self.sand[y][x] > 3 {
                    self.stack.push((x, y))
                }
            }
        }
    }

    fn topple(&mut self) {
        if self.is_relaxed() {
            return;
        }

        let FIELD_SIZE =  self.field.size.0 * self.field.size.1;
        let relaxation_strategy = self.relaxation_strategy.clone();
        /*
        match self.relaxation_strategy {
            ref ArrayTraversing => println!("1"),
            ref Stack => println!("2")
        };
        */

        match relaxation_strategy {
            RelaxationStrategy::ArrayTraversing => {
                let mut topplings_count = 0;
                for x in 0..self.field.size.0 {
                    for y in 0..self.field.size.1 {
                        if self.field.is_inside((x, y)) && self.sand[y][x] > 3 {
                            topplings_count += 1;
                            self.sand[y][x] -= 4;

                            for (x_neighbor, y_neighbor) in self.field.neighbours[y][x].iter() {
                                self.sand[*y_neighbor][*x_neighbor] += 1;
                            }
                        }
                    }
                }

                // println!("ArrayTraversing {} {}", topplings_count, FIELD_SIZE);
                self.is_relaxed = topplings_count == 0;

                if topplings_count < FIELD_SIZE / 8 {
                    println!("Change strategy to Stack {} {}", topplings_count, FIELD_SIZE);
                    self.build_stack();
                    self.relaxation_strategy = RelaxationStrategy::Stack;
                }
            },
            RelaxationStrategy::Stack => {

                // println!("Stack");
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
                self.is_relaxed = self.stack.is_empty();

                if self.stack.len() > FIELD_SIZE {
                    self.relaxation_strategy = RelaxationStrategy::ArrayTraversing;
                }
            }
        }
    }


    fn relax(&mut self) {
        while ! self.is_relaxed() {
            self.topple();
        }
    }

    fn is_relaxed(&self) -> bool {
        self.is_relaxed
    }

    fn add_grains(&mut self, (x, y): (usize, usize), count: u32) {
        if self.field.is_inside((x, y)) {
            self.sand[y][x] += count;
            if self.sand[y][x] > 3 {
                self.stack.push((x, y));
                self.is_relaxed = false;
            }
        }
    }

}


use std::ops;
impl<'a> ops::Add for &'a SandPile {
    type Output = SandPile;

    fn add(self, other: &'a SandPile) -> SandPile {
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

impl<'a> ops::Sub for &'a SandPile {
    type Output = SandPile;

    fn sub(self, other: &'a SandPile) -> SandPile {
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


        const BLUE3: [f32; 4] = [0.0, 0.0, 0.8, 1.0];
        const BLUE2: [f32; 4] = [0.2, 0.2, 1.0, 1.0];
        const BLUE1: [f32; 4] = [0.6, 0.6, 1.0, 1.0];


        let square_size: f64 = ((args.height as f64) / (sand.len() as f64))
                                    .min((args.width as f64) / (sand[0].len() as f64));

        let square = rectangle::square(0.0, 0.0, square_size);


        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(WHITE, gl);
            for (y, line) in sand.iter().enumerate() {
                for (x, cell) in line.iter().enumerate() {
                    let colour = match *cell  {
                        0 => WHITE,
                        1 => BLUE1,
                        2 => BLUE2,
                        3 => BLUE3,
                        _ => BLACK
                    };
                    let transform = c.transform.trans(square_size*(x as f64), square_size*(y as f64));

                    rectangle(colour, square, transform, gl);
                }
            }
        });
    }
}


pub struct Visualizer {
    app: App,
    window: Window,
}

impl Visualizer {
    fn visualize_relaxation(&mut self, sand_pile: &mut SandPile) {
        let mut events = Events::new(EventSettings::new());

        while ! sand_pile.is_relaxed() {
            if let Some(e) = events.next(&mut self.window) {
                if let Some(r) = e.render_args() {
                    self.app.render(&r, &sand_pile.sand);
                }
            }
            sand_pile.topple();
        }
    }

    fn loop_draw(&mut self, sand_pile: &SandPile) {
        let mut events = Events::new(EventSettings::new());
        while let Some(e) = events.next(&mut self.window) {
            if let Some(r) = e.render_args() {
                self.app.render(&r, &sand_pile.sand);
            }
        }
    }
}

fn compute_identity(field: &Field, visualizer: &mut Visualizer) {
    let mut sand_pile1 = SandPile::new(field.clone(), 6);
    let mut sand_pile2 = SandPile::new(field.clone(), 6);

    visualizer.visualize_relaxation(&mut sand_pile1);
    sand_pile2 = &sand_pile2 - &sand_pile1;
    visualizer.visualize_relaxation(&mut sand_pile2);
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

    let mut app = App {
        gl: GlGraphics::new(opengl),
    };

    let mut visualizer = Visualizer {app, window};

    let field = Field::square(400);

    compute_identity(&field, &mut visualizer);


    /*
    let mut sand_pile = SandPile::new(field.clone(), 0);
    sand_pile.add_grains((100, 100), 2);

    // 2^32 grains in center
    for i in 0..32 {
        println!("{}", i);
        sand_pile = &sand_pile + &sand_pile;
        visualizer.visualize_relaxation(&mut sand_pile);
    }
    */


    //sand_pile = sand_pile - sand_pile1;

    // visualizer.visualize_relaxation(&mut sand_pile);

    // visualizer.loop_draw(&sand_pile);
}