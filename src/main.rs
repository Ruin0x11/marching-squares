#![feature(core)]
extern crate rand;
extern crate noise;
extern crate rustbox;

use std::default::Default;

use noise::{perlin2, cell2_value, Brownian2, Seed};
use rustbox::{RustBox, Event, Key, Color};

mod noisefield;
use noisefield::NoiseField;

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

fn main() {
    let rb = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    // set up noisefield
    let mut field: NoiseField<f32> = NoiseField::new(Seed::new(0));
    field.add_noise(Box::new(perlin2));
    field.add_noise(Box::new(|seed, &[x, y]| cell2_value(seed, &[x, y]) / 2.0));
    field.add_noise(Box::new(Brownian2::new(perlin2, 5).wavelength(3.0)));
    
    let mut running = true;
    let mut unicode = false;
    let mut step = 0.1f32;
    let mut threshold = 0.4;
    let (mut startx, mut starty) = (0.0f32, 0.0f32);

    while running {
        let (rows, cols) = (rb.height(), rb.width() / 3);
        let (mut x, mut y) = (startx - cols as f32 * step / 2.0,
                              starty - rows as f32 * step / 2.0);

        for oy in 0..rows {
            for ox in 0..cols {
                let points = corners(x, y, step);
                let samples = [field.sample(&points[0]),
                               field.sample(&points[1]),
                               field.sample(&points[2]),
                               field.sample(&points[3])];

                rb.print(ox * 3, oy,
                         rustbox::RB_NORMAL, Color::White, Color::Black, 
                         march(&samples, threshold, unicode));
                x += step;
            }
            y += step;
            x = startx - cols as f32 * step / 2.0;
        }

        rb.present();

        if let Ok(Event::KeyEvent(Some(key))) = rb.poll_event(false) {
            match key {
                Key::Char('w') | Key::Up    => starty -= step,
                Key::Char('s') | Key::Down  => starty += step,
                Key::Char('a') | Key::Left  => startx -= step,
                Key::Char('d') | Key::Right => startx += step,

                Key::Char('+') if step > 0.01 => step -= 0.001,
                Key::Char('-') => step += 0.001,
                Key::Char('[') => threshold -= 0.01,
                Key::Char(']') => threshold += 0.01,

                Key::Char('u') => unicode = !unicode,

                Key::Esc | Key::Char('q') => running = false,

                _ => {}
            }
        }
    }
}
