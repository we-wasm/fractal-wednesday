// complex coord <-> buffers
use crate::render::Complex;
use std::{mem, slice};

pub struct TileBuffer<T> {
    w: usize,
    h: usize,
    buf: Vec<T>,
}

impl<T> TileBuffer<T>
where
    T: Clone,
{
    pub fn with_size(width: usize, height: usize, fill: T) -> Self {
        TileBuffer {
            w: width,
            h: height,
            buf: vec![fill; (width * height) as usize],
        }
    }

    pub fn resize(self, width: usize, height: usize, fill: T) -> TileBuffer<T> {
        if self.w == width && self.h == height {
            self
        } else {
            Self::with_size(width, height, fill)
        }
    }

    pub fn get_mut_buf(&mut self) -> &mut [u8] {
        // the buf is exactly the same as the expected array of bytes
        // just represented in chunks of 4
        unsafe {
            let u8_ptr = &mut *(self.buf.as_mut_ptr() as *mut T as *mut u8);
            slice::from_raw_parts_mut(u8_ptr, self.w * self.h * mem::size_of::<T>())
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.h as f32 / self.w as f32
    }

    pub fn apply<F>(&mut self, center: Complex, width: f64, f: F)
    where
        F: Fn(Complex) -> T,
    {
        let step = width / self.w as f64;
        let start_re = (center.re - width / 2.0) as f64;
        let start_im = (center.im - (width * (self.h as f64 / self.w as f64)) / 2.0) as f64;
        for y in 0..self.h {
            for x in 0..self.w {
                self.buf[(y * self.w + x) as usize] = f(Complex {
                    re: start_re + ((x as f64) * step),
                    im: start_im + ((y as f64) * step),
                });
            }
        }
    }
}

#[derive(Debug)]
pub struct Viewport {
    pub center: Complex,
    pub width: f64,
}

impl Viewport {
    pub fn translate(&mut self, u: f32, v: f32) {
        self.center = Complex {
            re: self.center.re + (self.width * u as f64),
            im: self.center.im + (self.width * v as f64),
        }
    }
    pub fn zoom(&mut self, z: f32, u: f32, v: f32) {
        let z = 1.0 + z;
        self.width = self.width * z as f64;
        self.translate((u - 0.5) - (u - 0.5) * z, (v - 0.5) - (v - 0.5) * z);
    }
}
