use minifb::{Key, Window, WindowOptions};
use std::collections::hash_set::HashSet;
use std::collections::hash_map::HashMap;

mod krand;
use krand::*;

#[derive(Copy, Clone, Debug)]
enum Command {
    PlayPause,
    Step,
    Faster,
    Slower,
    Reset,
    Next,
    Prev,
    Scramble,
}

struct Application {
    w: i32,
    h: i32,
    generation: i32,

    cells: Vec<u32>,

    play: bool,
    speed: i32,

    rule: u8,
}
const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const SCALE: usize = 4;

fn starting_cells(w: i32, h: i32) -> Vec<u32> {
    let mut cells = vec![0; w as usize * h as usize];
    cells[((w/2) + w * (h - 1)) as usize] = 0x00FFFFFFFF;
    cells
}
fn starting_cells_scrambled(seed: u32, w: i32, h: i32) -> Vec<u32> {
    let mut cells = vec![0; w as usize * h as usize];
    for i in 0..w {
        cells[(i + w * (h - 1)) as usize] = if chance(seed * 12351301 + i as u32, 0.5) {0x00FFFFFFFF} else {0};
    }
    cells
}

// add random bit flips

impl Application {
    fn new(w: i32, h: i32) -> Application {
        //let init = init_cells_nucleation_pt(w, h);
        Application {
            w,
            h,
            play: false,
            speed: 4,
            generation: 0,
            cells: starting_cells(w,h),
            rule: 110,
            
        }
    }


    fn apply_command(&mut self, command: Command) {
        println!("{:?}", command);

        match command {
            Command::PlayPause => self.play = !self.play,
            Command::Step => self.step(),
            Command::Faster => self.speed += 1,
            Command::Slower => self.speed = std::cmp::max(1, self.speed - 1),
            Command::Reset => {
                self.cells = starting_cells(self.w , self.h);
            },
            Command::Scramble => {
                self.cells = starting_cells_scrambled(self.generation as u32, self.w , self.h);
            },
            Command::Next => {
                self.rule += 1;
                println!("Rule {}", self.rule);
            },
            Command::Prev => {
                self.rule -= 1;
                println!("Rule {}", self.rule);
            },
        }
    }

    fn step(&mut self) {
        for j in 0..self.h-1 {
            for i in 0..self.w {
                self.cells[(i + self.w * j) as usize] = self.cells[(i + self.w * (j+1)) as usize];
            }
        }
        for i in 0..self.w {
            let neighbour = |x: i32| {
                let neighbour_x = (self.w + x + i) % self.w; // wraps
                let cell_val = self.cells[(neighbour_x + self.w * (self.h - 2)) as usize];
                cell_val == 0x00FFFFFFFF
            };

            let mut lookup = 0;
            if neighbour(-1) {
                lookup += 4;
            }
            if neighbour(0) {
                lookup += 2;
            }
            if neighbour(1) {
                lookup += 1;
            }
            let value = (self.rule >> lookup) & 1 == 1;
            
            self.cells[(i + self.w * (self.h - 1)) as usize] = if value {
                0x00FFFFFFFF
            } else {
                0x0000000000
            };
        }

        if chance(khash(self.generation as u32 * 12351234), 0.01) {
            let flip_x = khash((self.generation * 123151235) as u32) % self.w as u32;
            self.cells[(self.w * (self.h - 1) + flip_x as i32) as usize] ^= 0x00FFFFFFFF;
        }

        self.generation += 1;
    }
}



fn main() {
    let mut key_schema: HashMap<Key, Command> = HashMap::new();
    key_schema.insert(Key::Space, Command::PlayPause);
    key_schema.insert(Key::Period, Command::Step);
    key_schema.insert(Key::LeftBracket, Command::Slower);
    key_schema.insert(Key::RightBracket, Command::Faster);
    key_schema.insert(Key::R, Command::Reset);
    key_schema.insert(Key::S, Command::Scramble);
    key_schema.insert(Key::O, Command::Prev);
    key_schema.insert(Key::P, Command::Next);

    let mut app = Application::new((WIDTH/SCALE) as i32, (HEIGHT/SCALE) as i32);


    let mut window = Window::new(
        "1DCA",
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
            .update_with_buffer(&app.cells, WIDTH/SCALE, HEIGHT/SCALE)
            .unwrap();
    }
}