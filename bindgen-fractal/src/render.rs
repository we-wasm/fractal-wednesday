// color <- z=z**2+c <- complex coord

use std::ops::Add;

// Complex coordination
// https://rustwasm.github.io/wasm-bindgen/examples/julia.html
#[derive(Clone, Copy, Debug)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
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
pub fn mandel_iter(max_iter: u64, c: Complex) -> u64 {
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
#[repr(C)]
pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
    _a: u8,
}

fn tween_one(progress: i32, from: u8, to: u8) -> u8 {
    let from = from as i32;
    let to = to as i32;
    (from + (to - from) * progress / 255) as u8
}

impl RGB {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, _a: 255 }
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
    _a: 255,
};

pub fn build_palette(gradients: &Vec<[&RGB; 2]>, steps_per_grad: usize) -> Vec<RGB> {
    let num_gradients = gradients.len();
    let mut palette = Vec::with_capacity(num_gradients * steps_per_grad);
    for i in 0..num_gradients {
        let color = gradients[i][0];
        let next_color = gradients[i][1];
        for step in 0..steps_per_grad {
            let progress = (step * 255 / steps_per_grad) as i32;
            palette.push(color.tween(progress, next_color));
        }
    }
    palette
}

pub fn mandel_color(i: u64, palette: &Vec<RGB>) -> RGB {
    if i == 0 {
        BOTTOM
    } else {
        // This is on the hot loop, can len be removed?
        palette[(i % palette.len() as u64) as usize]
    }
}
