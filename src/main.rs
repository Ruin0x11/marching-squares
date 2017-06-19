extern crate rand;
extern crate noise;
extern crate rustbox;
#[macro_use] extern crate hlua;

use std::default::Default;
use std::fs::File;
use std::collections::HashMap;
use std::io::Read;

use rand::Rng;
use rand::distributions::{Weighted, WeightedChoice, IndependentSample};
use rustbox::{RustBox, Event, Key, Color};
use hlua::Lua;

mod noisefield;
use noisefield::NOISE;

type Cell = &'static str;

const CASES: [Cell; 16] = [
    r"   ",
    r"_  ",
    r"  _",
    r"___",
    r" \_",
    r"/ _",
    r" | ",
    r"_/ ",
    r"_/ ",
    r" | ",
    r"_ \",
    r"  \",
    r"___",
    r"  _",
    r"_  ",
    r"###",
];

const UNICODE_CASES: [Cell; 16] = [
    r"   ",
    r"─╮ ",
    r" ╭─",
    r"───",
    r" ╰─",
    r"╱▞╱",
    r" │█",
    r"╱▟█",
    r"─╯ ",
    r"█│ ",
    r"╲▚╲",
    r"█▙╲",
    r"───",
    r"█▛╱",
    r"╲▜█",
    r"███",
];

fn corners(x: f32, y: f32, width: f32) -> [[f32; 2]; 4] {
    let w = width / 2.0;
    [[x - w, y - w], [x + w, y - w], [x + w, y + w], [x - w, y + w]]
}

fn march(samples: &[f32; 4], threshold: f32, unicode: bool) -> Cell {
    let bits: Vec<usize> = samples.iter().map(|&s| if s > threshold { 1 } else { 0 }).collect();
    let case = bits[0] << 3 | bits[1] << 2 | bits[2] << 1 | bits[3];
    if unicode { UNICODE_CASES[case] } else { CASES[case] }
}

fn color(sample: f32) -> Color {
    if sample < 1.0/7.0 {
        Color::Red
    } else if sample < 2.0/7.0 {
        Color::Green
    } else if sample < 3.0/7.0 {
        Color::Yellow
    } else if sample < 4.0/7.0 {
        Color::Blue
    } else if sample < 5.0/7.0 {
        Color::Magenta
    } else if sample < 6.0/7.0 {
        Color::Cyan
    } else {
        Color::White
    }
}

pub fn run_script<'a, 'lua>(lua: &'a mut Lua<'lua>, filename: &str) -> Result<(), hlua::LuaError>
{
    let mut script = String::new();
    let full_path = format!("data/{}.lua", filename);
    File::open(full_path).expect("No such script file").read_to_string(&mut script).unwrap();
    lua.execute::<()>(&script)
}

fn main() {
    let rb = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    let mut lua = Lua::new();
    lua.openlibs();
    noisefield::add_lua_interop(&mut lua);

    let mut running = true;
    let mut unicode = false;
    let mut update = true;
    let mut step = 0.1f32;
    let mut threshold = 0.4;
    let (mut startx, mut starty) = (0.0f32, 0.0f32);

    while running {
        if update {
            match run_script(&mut lua, "map") {
                Ok(()) => {
                    let (cols, rows) = (rb.width() / 3, rb.height());
                    let (mut x, mut y) = (startx - cols as f32 * step / 2.0,
                                          starty - rows as f32 * step / 2.0);

                    let mut v: HashMap<(usize, usize), &str> = HashMap::new();

                    for oy in 0..rows {
                        for ox in 0..cols {
                            let points = corners(x, y, step);
                            let samples = NOISE.with(|n|{
                                let nb = n.borrow();
                                [nb.sample(points[0]),
                                 nb.sample(points[1]),
                                 nb.sample(points[2]),
                                 nb.sample(points[3])]
                            });

                            v.insert((ox, oy), march(&samples, threshold, unicode));

                            x += step;
                        }
                        y += step;
                        x = startx - cols as f32 * step / 2.0;
                    }

                    match run_script(&mut lua, "biome") {
                        Ok(()) => {
                    let (mut x, mut y) = (startx - cols as f32 * step / 2.0,
                                          starty - rows as f32 * step / 2.0);
                            for oy in 0..rows {
                                for ox in 0..cols {
                                    let points = corners(x, y, step);
                                    let sample = NOISE.with(|n|{
                                        let nb = n.borrow();
                                        nb.sample([x, y])
                                    });

                                    let color = color(sample);

                                    rb.print(ox * 3, oy,
                                             rustbox::RB_NORMAL, color, Color::Black,
                                             v[&(ox, oy)]);

                                    x += step;
                                }
                                y += step;
                                x = startx - cols as f32 * step / 2.0;
                            }
                        },
                        Err(e) => rb.print(0, 0,
                                           rustbox::RB_NORMAL, Color::White, Color::Black,
                                           &format!("{:?}", e)),
                    }
                }
                Err(e) => rb.print(0, 0,
                                   rustbox::RB_NORMAL, Color::White, Color::Black,
                                   &format!("{:?}", e)),

            }
            rb.present();
        }

        if let Ok(Event::KeyEvent(key)) = rb.poll_event(false) {
            match key {
                Key::Char('w') | Key::Up    => starty -= step,
                Key::Char('s') | Key::Down  => starty += step,
                Key::Char('a') | Key::Left  => startx -= step,
                Key::Char('d') | Key::Right => startx += step,

                Key::Char('=') if step > 0.01 => step -= 0.001,
                Key::Char('-') => step += 0.001,
                Key::Char('[') => threshold -= 0.01,
                Key::Char(']') => threshold += 0.01,

                Key::Char('u') => unicode = !unicode,

                Key::Esc | Key::Char('q') => running = false,

                _ => {}
            }
            update = true;
        }
    }
}
