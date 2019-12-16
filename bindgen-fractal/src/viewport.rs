// complex coord <-> buffers
use crate::render::Complex;
use std::{mem, slice};

/// A coordinate on a buffer in the range 0-1 for u and v
pub struct TexCoord {
    pub u: f32,
    pub v: f32,
}

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

    pub fn apply<F>(&mut self, start: TexCoord, end: TexCoord, f: F)
    where
        F: Fn(TexCoord) -> T,
    {
        let start_x = (start.u * self.w as f32) as usize;
        let end_x = (end.u * self.w as f32) as usize;
        let start_y = (start.v * self.h as f32) as usize;
        let end_y = (end.v * self.h as f32) as usize;
        let step_u = (end.u - start.u) / (end_x as f32 - start_x as f32);
        let step_v = (end.v - start.v) / (end_y as f32 - start_y as f32);
        let mut u = start.u;
        let mut v = start.v;
        for y in start_y..end_y {
            let y_off = y * self.w;
            for x in start_x..end_x {
                self.buf[(y_off + x) as usize] = f(TexCoord { u, v });
                u += step_u;
            }
            u = 0.0;
            v += step_v;
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

    pub fn map_coord(&self, c: TexCoord, aspect_ratio: f32) -> Complex {
        let aspect_ratio = aspect_ratio as f64;
        Complex {
            re: (self.center.re - self.width / 2.0 + c.u as f64 * self.width) as f64,
            im: (self.center.im - self.width / 2.0 + c.v as f64 * self.width) as f64 * aspect_ratio,
        }
    }

    pub fn zoom(&mut self, z: f32, u: f32, v: f32) {
        let z = 1.0 + z;
        self.width = self.width * z as f64;
        self.translate((u - 0.5) - (u - 0.5) * z, (v - 0.5) - (v - 0.5) * z);
    }
}
