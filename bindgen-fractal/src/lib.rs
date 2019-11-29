extern crate wasm_bindgen;
extern crate wee_alloc;

use std::ops::Add;

use web_sys::{CanvasRenderingContext2d, ImageData};

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

pub struct TileBuffer {
    w: u32,
    h: u32,
    buf: Vec<RGB>,
}

impl TileBuffer {
    fn with_size(width: u32, height: u32) -> Self {
        TileBuffer {
            w: width,
            h: height,
            buf: Vec::with_capacity((width * height) as usize),
        }
    }
}

// Complex coordination
// https://rustwasm.github.io/wasm-bindgen/examples/julia.html
#[derive(Clone, Copy)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn square(self) -> Complex {
        let re = (self.re * self.re) - (self.im * self.im);
        let im = 2.0 * self.re * self.im;
        Complex { re, im }
    }

    fn norm(&self) -> f64 {
        (self.re * self.re) + (self.im * self.im)
    }
}

impl Add<Complex> for Complex {
    type Output = Complex;

    fn add(self, rhs: Complex) -> Complex {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

// Mandelbrot maths
fn mandel_iter(max_iter: u64, c: Complex) -> u64 {
    let mut z: Complex = c;

    let mut iter = 1;

    while (z.norm() <= 4.0) && (iter < max_iter) {
        z = c + z.square();
        iter = iter + 1;
    }

    if iter == max_iter {
        0
    } else {
        iter
    }
}

// With this byte order javascript can copy it straight into canvas
#[derive(Clone, Copy)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

fn tween_one(progress: i32, from: u8, to: u8) -> u8 {
    let from = from as i32;
    let to = to as i32;
    (from + (to - from) * progress / 255) as u8
}

impl RGB {
    fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    fn tween(&self, progress: i32, to: &RGB) -> RGB {
        RGB::rgb(
            tween_one(progress, self.r, to.r),
            tween_one(progress, self.g, to.g),
            tween_one(progress, self.b, to.b),
            // This invalid code saves 2k?
            // (to.r as u16 * 255 / progress) as u8,
        )
    }
}

static BOTTOM: RGB = RGB {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};

// golfing to do here...
fn build_palette(gradients: &Vec<[&RGB; 2]>, steps_per_grad: usize) -> Vec<RGB> {
    let num_gradients = gradients.len();
    let mut palette = Vec::with_capacity(num_gradients * steps_per_grad);
    for i in 0..num_gradients {
        let color = gradients[i][0];
        let next_color = gradients[i][1];
        for step in 0..steps_per_grad {
            let progress = (step * 255 / steps_per_grad) as i32;

            palette[i * steps_per_grad + step] = color.tween(progress, next_color);
        }
    }
    palette
}

fn mandel_color(i: u64, palette: &Vec<&RGB>) -> RGB {
    if i == 0 {
        BOTTOM
    } else {
        // This is on the hot loop, can len be removed?
        palette[(i % palette.len() as u64) as usize]
    }
}

// Javascript jams
#[wasm_bindgen]
pub extern "C" fn render(
    ctx: &CanvasRenderingContext2d,
    width: u32,
    height: u32,
    max_iter: u32,
    center_re: f32,
    center_im: f32,
    viewport_width: f32,
) -> Result<(), JsValue> {
    console_log!("Rendering a {}x{} fractal", width, height);
    let mut tile = TileBuffer::with_size(width, height);
    render_tile(&mut tile, max_iter, center_re, center_im, viewport_width);
    let data =
        ImageData::new_with_u8_clamped_array_and_sh(Clamped(tile.buf.as_mut_ptr()), width, height)?;
    ctx.put_image_data(&data, 0.0, 0.0)
}

// We split this out so that we can escape 'unsafe' as quickly
// as possible.
fn render_tile(
    tile: &mut TileBuffer,
    max_iter: u32,
    center_re: f32,
    center_im: f32,
    viewport_width: f32,
) {
    let width = tile.w;
    let height = tile.h;

    let step = (viewport_width / width as f32) as f64;
    let start_re = (center_re - viewport_width / 2.0) as f64;
    let start_im = (center_im - (viewport_width * (height as f32 / width as f32)) / 2.0) as f64;
    let blue = RGB::rgb(0, 183, 255);
    let orange = RGB::rgb(255, 128, 0);
    let black = RGB::rgb(0, 0, 0);
    let white = RGB::rgb(255, 255, 255);

    let gradients = vec![
        [&black, &blue],
        [&blue, &white],
        [&white, &orange],
        [&orange, &black],
    ];

    let palette = build_palette(&gradients, 4);

    for y in 0..height {
        for x in 0..width {
            let c = mandel_color(
                mandel_iter(
                    max_iter as u64,
                    Complex {
                        re: start_re + ((x as f64) * step),
                        im: start_im + ((y as f64) * step),
                    },
                ),
                &palette,
            );
            tile.buf[(y * width + x) as usize] = c;
        }
    }
}
