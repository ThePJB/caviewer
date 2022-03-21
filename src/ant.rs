use minifb::{Key, Window, WindowOptions};
use std::collections::hash_set::HashSet;
use std::collections::hash_map::HashMap;

mod krand;
use krand::*;

/*
How to enumerate rules
 - can just count

R

RR
RL

RRR
RRL
RLL

RRRR
RRRL
RRLR
RLRR
RLRL
RLLR
RLLL


relatives:
 - negation (identical). if we ignore half we ignore all negations, ie append implicit 1, or count the 1
 - rotation
 - reversal (any relation?)
 - periodic

classification:
 - growth per generation
 - histogram - should show highways, other structures
 - symmetry - squares and triangles
 - boundary hecticness
 - 'eccentricity'
 - highways should be easy

theorems:
 - symmetry: circularly paired LLs RRs
 - prefix does what
 - suffix does what
 - ratio Ls to Rs?

can i count the opposite way, similar prefixes would occur together
also count shift left / shift right

cool findings
RRLLLRRL shuriken
LRLLLRRL simple square
RLLLLRRL complex square - go between them its a sick turing machine. the way it climbs along rails

this but with extra L is cool too

LLRLRRLL thicc highway

RLRRRLLL > 
LLRRRLLL is sick as fuck

LLLLRLLL > RRRRLLLL also sick as fuck. carnage. mold


highwaying + chaos mad structure, repititions

what about increment rule on 0

suffix families, eg add more RLs. so many ways to enumerate
RRLLLRLRL

dragon curvey
RLRLRLLRLRLRLRLRLRLRLRLRL

"first this then infinite of this"

LRLLL<anything> makes the square
start with some RLs and then a bunch of Rs
aRLnR

sierpinski triangles are RRLLLRLLLL... finish the series
prepend L, triangle highway potential

how about chucking in dragon curve

infinite trailing Ls for instance
*/

#[derive(Copy, Clone, Debug)]
enum Command {
    PlayPause,
    Step,
    Faster,
    Slower,
    Reset,
    Next,
    Prev,
    NextLSH,
    PrevRSH,
}

struct Application {
    w: i32,
    h: i32,
    generation: i32,

    visited_cells: HashMap<(i32,i32), u8>,
    ant_pos: (i32, i32),
    ant_dir: (i32, i32),

    play: bool,
    speed: i32,

    dirty: bool,

    rule: u64,
    rule_len: u8,
}

impl Application {
    fn new(w: i32, h: i32) -> Application {
        Application {
            w,
            h,
            play: false,
            speed: 10000,
            dirty: true,
            generation: 0,
            visited_cells: HashMap::new(),
            ant_pos: (0,0),
            ant_dir: (1,0),
            rule: 0b10,
            rule_len: 2,
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
            Command::Next => {
                self.rule += 1;
                println!("Rule {}", rule_string(self.rule));
                self.rule_len = (64 - self.rule.leading_zeros()) as u8;
            },
            Command::Prev => {
                self.rule -= 1;
                println!("Rule {}", rule_string(self.rule));
                self.rule_len = (64 - self.rule.leading_zeros()) as u8;
            },
            Command::NextLSH => {
                self.rule <<= 1;
                println!("Rule {}", rule_string(self.rule));
                self.rule_len = (64 - self.rule.leading_zeros()) as u8;
            },
            Command::PrevRSH => {
                self.rule >>= 1;
                println!("Rule {}", rule_string(self.rule));
                self.rule_len = (64 - self.rule.leading_zeros()) as u8;
            },
            Command::Reset => {
                self.play = false;
                self.visited_cells = HashMap::new();
                self.ant_pos = (0,0);
                self.ant_dir = (1,0);
            },
        }
    }

    fn step(&mut self) {
        // println!("step");
        self.dirty = true;

        self.ant_pos.0 += self.ant_dir.0;
        self.ant_pos.1 += self.ant_dir.1;

        let e = self.visited_cells.entry(self.ant_pos).or_insert(0);
        
        let cell_num = *e % self.rule_len;
        *e += 1;
        let rot_left = self.rule >> cell_num & 1 == 1;
        self.ant_dir = if rot_left {
            (-self.ant_dir.1, self.ant_dir.0)
        } else {
            (self.ant_dir.1, -self.ant_dir.0)
        };

        self.generation += 1;
    }

    // just this
    fn draw_to_buffer(&mut self, px_buf: &mut Vec<u32>) {
        if !self.dirty {
            return
        }
        self.dirty = false;

        for px_i in 0..self.w {
            let cell_i = px_i - self.w/2;// + self.ant_pos.0;
            for px_j in 0..self.h {
                let cell_j = px_j - self.h/2;// + self.ant_pos.1;
                let cell_num = *self.visited_cells.get(&(cell_i, cell_j)).unwrap_or(&0) % self.rule_len;
                let colour_strength = cell_num as f32 / (self.rule_len - 1) as f32;
                px_buf[(px_i * self.w + px_j) as usize] = marshal_colour(colour_strength, colour_strength, colour_strength);
            }
        }
    }
}

fn marshal_colour(r: f32, g: f32, b: f32) -> u32 {
    ((255.0 * r) as u32) << 16 | 
    ((255.0 * g) as u32) << 8 | 
    ((255.0 * b) as u32)
}

fn rule_string(mut rule: u64) -> String {
    let mut s = String::new();

    while rule != 0 {
        s.push(if rule & 1 == 1 {'L'} else {'R'});
        rule >>= 1;
    }
    s
}

#[test]
fn tmc() {
    assert_eq!(marshal_colour(1.0, 1.0, 1.0), 0x00FFFFFF);
}

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const SCALE: usize = 1;

fn main() {
    let mut key_schema: HashMap<Key, Command> = HashMap::new();
    key_schema.insert(Key::Space, Command::PlayPause);
    key_schema.insert(Key::Period, Command::Step);
    key_schema.insert(Key::LeftBracket, Command::Slower);
    key_schema.insert(Key::RightBracket, Command::Faster);
    key_schema.insert(Key::R, Command::Reset);
    key_schema.insert(Key::M, Command::Next);
    key_schema.insert(Key::N, Command::Prev);
    key_schema.insert(Key::K, Command::NextLSH);
    key_schema.insert(Key::J, Command::PrevRSH);

    let mut app = Application::new((WIDTH/SCALE) as i32, (HEIGHT/SCALE) as i32);

    let mut buffer: Vec<u32> = vec![0; WIDTH/SCALE * HEIGHT/SCALE];

    let mut window = Window::new(
        "ant",
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
            .update_with_buffer(&buffer, WIDTH/SCALE, HEIGHT/SCALE)
            .unwrap();
    }
}