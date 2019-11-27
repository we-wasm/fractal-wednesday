#![feature(alloc_error_handler, lang_items)]
#![no_std]

extern crate alloc;
extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::f32;
use core::ops::Add;

// Compiler calming
#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn error_handler(_: core::alloc::Layout) -> ! {
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

extern "C" {
    fn js_logu(x: u32);
    fn js_logf(x: f32);
}

// Memory Management
#[repr(C)]
pub struct TileBuffer {
    w: u32,
    h: u32,
    buf: Vec<u32>,
}

// http://jakegoulding.com/rust-ffi-omnibus/objects/
#[no_mangle]
pub extern "C" fn alloc_tile(width: u32, height: u32) -> *mut TileBuffer {
    let tile = TileBuffer {
        w: width,
        h: height,
        buf: Vec::with_capacity((width * height) as usize),
    };
    Box::into_raw(Box::new(tile))
}

#[no_mangle]
pub extern "C" fn get_buffer(tile_ptr: *mut TileBuffer) -> *mut u32 {
    let tile = unsafe {
        assert!(!tile_ptr.is_null());
        &mut *tile_ptr
    };

    tile.buf.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn free_tile(tile_ptr: *mut TileBuffer) {
    if tile_ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(tile_ptr);
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

const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    255 << 24 | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
}

fn rgbf(r: f32, g: f32, b: f32) -> u32 {
    rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn scale_value(value: f32, v_low: f32, v_high: f32, s_low: f32, s_high: f32) -> f32 {
    (((value - v_low) / (v_high - v_low)) * (s_high - s_low)) + s_low
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> u32 {
    let i = (h * 6.0) as u8 as f32;
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    if h < 0.16666666666 {
        rgbf(v, t, p)
    } else if h < 0.33333333333 {
        rgbf(q, v, p)
    } else if h < 0.5 {
        rgbf(p, v, t)
    } else if h < 0.66666666666 {
        rgbf(p, q, v)
    } else if h < 0.83333333333 {
        rgbf(t, p, v)
    } else {
        rgbf(v, p, q)
    }
}

static BOTTOM: u32 = rgb(0, 0, 0);

// golfing to do here...
fn build_palette(size: usize) -> Vec<u32> {
    let mut palette = Vec::with_capacity(size);

    for i in 0..size {
        let m = i as f32 / size as f32;
        palette.push(if m < 0.5 {
            let h = 0.66666666666; // blue
                                   // black to white
            let v = scale_value(m, 0.0, 0.5, 0.0, 1.0);
            hsv_to_rgb(h, 0.5, v)
        } else {
            let h = 0.08333333333; // orange
            let v = scale_value(m, 0.5, 1.0, 1.0, 0.0);
            hsv_to_rgb(h, 0.5, v)
        });
    }
    palette
}

fn mandel_color(i: u64, palette: &Vec<u32>) -> u32 {
    if i == 0 {
        BOTTOM
    } else {
        palette[(i % palette.len() as u64) as usize]
    }
}

// Javascript jams
#[no_mangle]
pub extern "C" fn render(
    tile_ptr: *mut TileBuffer,
    max_iter: u32,
    center_re: f32,
    center_im: f32,
    viewport_width: f32,
) {
    let tile = unsafe {
        assert!(!tile_ptr.is_null());
        &mut *tile_ptr
    };
    render_frame_safe(tile, max_iter, center_re, center_im, viewport_width)
}

// We split this out so that we can escape 'unsafe' as quickly
// as possible.
fn render_frame_safe(
    tile: &mut TileBuffer,
    max_iter: u32,
    center_re: f32,
    center_im: f32,
    viewport_width: f32,
) {
    let width = tile.w;
    let height = tile.h;
    let buf = tile.buf.as_mut_ptr();

    let step = (viewport_width / width as f32) as f64;
    let start_re = (center_re - viewport_width / 2.0) as f64;
    let start_im = (center_im - viewport_width / 2.0) as f64;

    let palette = build_palette(20);
    unsafe {
        js_logf(1.6);
        js_logu(palette[8]);
    }

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
            // saves 1.5k of code
            unsafe { *buf.offset((y * width + x) as isize) = c }
            // tile.buf[(y * width + x) as usize] = c;
        }
    }
}
