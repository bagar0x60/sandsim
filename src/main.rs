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

    fn ring(outer_radius: usize, inner_radius: usize) -> Field {
        let size = (2*outer_radius + 1, 2*outer_radius + 1);

        let mut interior = vec![vec![false; 2*outer_radius + 1]; 2*outer_radius + 1];

        let outer_radius = outer_radius as i32;
        let inner_radius = inner_radius as i32;
        let (x_center, y_center) = (outer_radius, outer_radius);

        for x in 0..(2*outer_radius + 1) {
            for y in 0..(2*outer_radius + 1) {
                interior[y as usize][x as usize] =
                        (x - x_center).pow(2) + (y - y_center).pow(2) <= outer_radius.pow(2) &&
                        (x - x_center).pow(2) + (y - y_center).pow(2) >= inner_radius.pow(2);
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

// todo rewrite as generic
struct BufferedStack {
    stacks: Vec<Vec<(usize, usize)>>,
    stacks_tops: Vec<usize>,
    current_stack: usize,
    next_stack: usize,
}

impl BufferedStack {
    fn new(stacks_count: usize) -> BufferedStack {
        assert_ne!(stacks_count, 0);
        let stacks: Vec<Vec<(usize, usize)>> = vec![Vec::new(); stacks_count];
        let stacks_tops: Vec<usize> = vec![0; stacks_count];
        let current_stack = 0;
        let next_stack = (current_stack + 1) % stacks.len();
        BufferedStack{stacks, stacks_tops, current_stack, next_stack}
    }

    fn push(&mut self, (x, y): (usize, usize), stack_index: usize) {
        let top = self.stacks_tops[stack_index];
        let stack = &mut self.stacks[stack_index];
        if top >= stack.len() {
            stack.push((x, y));
            // println!("{} {}", self.next_stack, top);
        } else {
            stack[top] = (x, y);
        }
        self.stacks_tops[stack_index] += 1;
        // println!("push {:?} {:?}", stack, (x, y));
    }

    fn push_next(&mut self, (x, y): (usize, usize)) {
        let stack_index = self.next_stack;
        self.push((x, y), stack_index);
    }

    fn push_current(&mut self, (x, y): (usize, usize)) {
        let stack_index = self.current_stack;
        self.push((x, y), stack_index);
    }

    fn pop(&mut self, stack_index: usize) -> Option<(usize, usize)> {
        let top = self.stacks_tops[stack_index];
        let stack = &mut self.stacks[stack_index];
        // println!("pop {:?}. {:?}", stack, top);

        if top == 0 {
            None
        } else {
            self.stacks_tops[stack_index] -= 1;
            Some(stack[top - 1])
        }
    }

    fn pop_current(&mut self) -> Option<(usize, usize)> {
        let stack_index = self.current_stack;
        self.pop(stack_index)
    }

    fn pop_next(&mut self) -> Option<(usize, usize)> {
        let stack_index = self.next_stack;
        self.pop(stack_index)
    }

    fn is_empty_current(&self) -> bool {
        self.stacks_tops[self.current_stack] == 0
    }

    fn is_empty_next(&self) -> bool {
        self.stacks_tops[self.next_stack] == 0
    }

    fn swap(&mut self) {
        self.current_stack = self.next_stack;
        self.next_stack = (self.current_stack + 1) % self.stacks.len()
    }
}



pub struct SandPile {
    field: Field,
    sand: Vec<Vec<u32>>,
    stack: BufferedStack,
    is_relaxed: bool,
    in_stack: Vec<Vec<bool>>,
}

impl SandPile {
    fn new(field: Field, sand_level: u32) -> SandPile {
        let (x_max, y_max) = field.size;
        let sand = vec![vec![sand_level; x_max]; y_max];
        let mut in_stack = vec![vec![false; x_max]; y_max];
        let mut stack = BufferedStack::new(2);
        let is_relaxed = sand_level <= 3;
        if ! is_relaxed {
            for x in 0..x_max {
                for y in 0..y_max {
                    if field.is_inside((x, y)) {
                        stack.push_current((x, y));
                        in_stack[y][x] = true;
                    }
                }
            }
        }


        SandPile {field, sand, stack, is_relaxed, in_stack}
    }

    fn topple(&mut self, topples_lim: u64) {
        if self.is_relaxed() {
            return;
        }

        // println!("topple");
        let mut topples_count: u64 = 0;

        while let Some((x, y)) = self.stack.pop_current() {
            if self.sand[y][x] > 3 {
                self.sand[y][x] -= 4;
                topples_count += 1;

                if self.sand[y][x] > 3 {
                    self.stack.push_current((x, y));
                } else {
                    self.in_stack[y][x] = false;
                }

                for (x_neighbor, y_neighbor) in self.field.neighbours[y][x].iter() {
                    self.sand[*y_neighbor][*x_neighbor] += 1;
                    if  self.sand[*y_neighbor][*x_neighbor] > 3 &&
                        ! self.in_stack[*y_neighbor][*x_neighbor] {
                        self.stack.push_current((*x_neighbor, *y_neighbor));
                        self.in_stack[*y_neighbor][*x_neighbor] = true;
                    }
                }

                if topples_count > topples_lim {
                    break;
                }
            }
        }

        // self.stack.swap();
        self.is_relaxed = self.stack.is_empty_current();
    }


    fn relax(&mut self) {
        while ! self.is_relaxed() {
            self.topple(100);
        }
    }

    fn is_relaxed(&self) -> bool {
        self.is_relaxed
    }

    fn add_grains(&mut self, (x, y): (usize, usize), count: u32) {
        if self.field.is_inside((x, y)) {
            self.sand[y][x] += count;
            if self.sand[y][x] > 3 && ! self.in_stack[y][x] {
                self.in_stack[y][x] = true;
                self.stack.push_current((x, y));
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
            sand_pile.topple(1_000_000);
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
    visualizer.loop_draw(&sand_pile2);
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

    let field = Field::square(300);

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