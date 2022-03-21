use minifb::{Key, Window, WindowOptions};
use std::collections::hash_set::HashSet;
use std::collections::hash_map::HashMap;

mod krand;
use krand::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Cell {
    Rock,
    Paper,
    Scissors,
    Antirock,
}

fn beats(c1: Cell, c2: Cell) -> bool {
    match (c1, c2) {
        (Cell::Rock, Cell::Scissors) => true,
        (Cell::Paper, Cell::Rock) => true,
        (Cell::Scissors, Cell::Paper) => true,
        (Cell::Antirock, Cell::Rock) => true,
        (Cell::Paper, Cell::Antirock) => true,
        (Cell::Scissors, Cell::Antirock) => true,
        _ => false,
    }
}

#[derive(Copy, Clone, Debug)]
enum Command {
    PlayPause,
    Step,
    Faster,
    Slower,
    Reset,
}

struct Application {
    w: i32,
    h: i32,
    scale: i32,
    generation: i32,

    initial_state: Vec<Cell>,
    cells: [Vec<Cell>; 2],
    curr_buffer: usize,

    play: bool,
    speed: i32,

    dirty: bool,
}

fn init_cells_random(w: i32, h: i32) -> Vec<Cell> {
    let mut v = vec![Cell::Rock; (w*h) as usize];
    for i in 0..w*h {
        v[i as usize] = match khash(i as u32) % 3 {
            0 => Cell::Rock,
            1 => Cell::Paper,
            2 => Cell::Scissors,
            _ => panic!("unreachable"),
        };
    }
    v
}


fn init_cells_nucleation_pt(w: i32, h: i32) -> Vec<Cell> {
    let mut v = vec![Cell::Rock; (w*h) as usize];

    for i in 0..w {
        for j in 0..h {
            if i < w/2 {
                if j < h/2 {
                    v[(i*w + j) as usize] = Cell::Paper;
                } else {
                    v[(i*w + j) as usize] = Cell::Scissors;
                }
            }
        }
    }

    //let i = (w/2 * h + h/2) as usize;
    //v[i] = Cell::Paper;
    //v[i+1] = Cell::Scissors;
    v
}

impl Application {
    fn new(w: i32, h: i32, scale: i32) -> Application {
        let init = init_cells_random(w, h);
        //let init = init_cells_nucleation_pt(w, h);
        Application {
            w,
            h,
            scale,
            initial_state: init.clone(),
            cells: [init.clone(), init.clone()],
            curr_buffer: 0,
            play: false,
            speed: 4,
            dirty: true,
            generation: 0,
        }
    }


    fn apply_command(&mut self, command: Command) {
        println!("{:?}", command);

        self.dirty = true;

        match command {
            Command::PlayPause => self.play = !self.play,
            Command::Step => self.step(),
            Command::Faster => self.speed += 1,
            Command::Slower => self.speed = std::cmp::max(1, self.speed - 1),
            Command::Reset => {
                self.play = false;
                self.cells[0] = self.initial_state.clone();
                self.cells[1] = self.initial_state.clone();
            },
        }
    }

    fn step(&mut self) {
        println!("step");
        self.dirty = true;
        //let beaten_threshold = 4;

        for i in 0..self.w {
            for j in 0..self.h {
/*
                let cosmic_ray_roll = rand::thread_rng().gen_range(0.0..1.0);
                if cosmic_ray_roll < 0.0001 {
                    //self.cells[1 - self.curr_buffer][(i*self.w + j) as usize] = Cell::Antirock;
                    self.cells[1 - self.curr_buffer][(i*self.w + j) as usize] = Cell::Rock;
                    /*
                    self.cells[1 - self.curr_buffer][(i*self.w + j) as usize] = match rand::thread_rng().gen_range(0..=2){
                        0 => Cell::Rock,
                        1 => Cell::Paper,
                        2 => Cell::Scissors,
                        _ => panic!("unreachable"),
                    };
                    */

                    continue;
                }
*/
                let neighbour_dirs = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
                let this_cell = self.cells[self.curr_buffer][(i*self.w + j) as usize];
                let mut beaten = 0;
                let beaten_threshold = if chance((self.generation + i * 314181 + j * 12135951) as u32, 0.01) {
                    3
                } else {
                    4
                };
                // let beaten_threshold = 3;
            
                
                let mut beater = Cell::Rock;
                for nd in neighbour_dirs {
                    let ip = i + nd.0;
                    let jp = j + nd.1;
                    if ip < 0 || ip >= self.w || jp < 0 || jp >= self.h { continue; }
                    let neighbour_cell = self.cells[self.curr_buffer][(ip*self.w + jp) as usize];
                    if beats(neighbour_cell, this_cell) {
                        beaten += 1;
                        beater = neighbour_cell;
                        if neighbour_cell == Cell::Antirock {
                            beaten += 10;
                            break;
                        }
                        if this_cell == Cell::Antirock {
                            beaten += 10;
                            break;
                        }
                    }
                }
                if beaten >= beaten_threshold {
                    self.cells[1 - self.curr_buffer][(i*self.w + j) as usize] = beater;
                } else {
                    self.cells[1 - self.curr_buffer][(i*self.w + j) as usize] = this_cell;
                }
            }
        }
        self.curr_buffer = 1 - self.curr_buffer;
        self.generation += 1;
    }

    fn draw_to_buffer(&mut self, px_buf: &mut Vec<u32>) {
        if !self.dirty {
            return
        }
        self.dirty = false;

        for cell_i in 0..self.w {
            for cell_j in 0..self.h {
                let this_cell = self.cells[self.curr_buffer][(cell_i * self.w + cell_j) as usize];
                let colour = match this_cell {
                    Cell::Rock => 0x00FF0000,
                    Cell::Paper => 0x0000FF00,
                    Cell::Scissors => 0x000000FF,
                    Cell::Antirock => 0x00FFFFFF,
                };
                for scale_i in 0..self.scale {
                    for scale_j in 0..self.scale {
                        px_buf[(((cell_i*self.scale) + scale_i) * (self.w*self.scale) + (cell_j*self.scale) + scale_j) as usize] = colour;

                    }
                }
            }
        }
    }
}

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const SCALE: usize = 4;

fn main() {
    let mut key_schema: HashMap<Key, Command> = HashMap::new();
    key_schema.insert(Key::Space, Command::PlayPause);
    key_schema.insert(Key::Period, Command::Step);
    key_schema.insert(Key::LeftBracket, Command::Slower);
    key_schema.insert(Key::RightBracket, Command::Faster);
    key_schema.insert(Key::R, Command::Reset);

    let mut app = Application::new((WIDTH/SCALE) as i32, (HEIGHT/SCALE) as i32, SCALE as i32);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "RPS",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    //window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut generation = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some(keys_pressed) = window.get_keys_pressed(minifb::KeyRepeat::Yes) {
            for key in keys_pressed.iter() {
                if let Some(command) = key_schema.get(key) {
                    app.apply_command(*command);
                }
            }
        }

        app.draw_to_buffer(&mut buffer);
        
        generation += 1;
        let fps_divide = if app.speed > 4 {
                1
            } else {
                30 / app.speed
            };
        let updates_per_frame = if app.speed > 4 {
                app.speed - 4
            } else {
                1
            };

        if app.play && generation % fps_divide == 0 {
            for _ in 0..updates_per_frame {
                app.step();
            }
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}