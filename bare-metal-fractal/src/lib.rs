#![feature(alloc_error_handler, lang_items)]
#![no_std]

extern crate alloc;
extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloc::alloc::alloc;
use alloc::boxed::Box;
use alloc::slice;
use core::alloc::Layout;
use core::f32;
use core::ffi::c_void;
use core::mem;
use core::ops::{Add, Deref, DerefMut};

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
    buf: BoxedSlice<u32>,
}

// Golfing away vec...
struct BoxedSlice<T>(Box<[T]>);

impl<T> BoxedSlice<T> {
    // copied from vec source at
    // https://github.com/rust-lang/rust/blob/master/src/liballoc/raw_vec.rs
    pub fn with_size(size: usize) -> Self {
        let elem_size = mem::size_of::<T>();
        let alloc_size = size * elem_size;
        let align = mem::align_of::<T>();
        unsafe {
            let layout = Layout::from_size_align_unchecked(alloc_size, align);
            let ptr = alloc(layout) as *mut T;
            let s = slice::from_raw_parts_mut(ptr, alloc_size);
            BoxedSlice(Box::from_raw(s))
        }
    }

    pub fn len(&self) -> usize {
        self.0.len() / mem::size_of::<T>()
    }
}
impl<T> Deref for BoxedSlice<T> {
    type Target = Box<[T]>;

    fn deref(&self) -> &Box<[T]> {
        &self.0
    }
}
impl<T> DerefMut for BoxedSlice<T> {
    fn deref_mut(&mut self) -> &mut Box<[T]> {
        &mut self.0
    }
}
// implementing a bounds unchecked slice access would shave some kb

// http://jakegoulding.com/rust-ffi-omnibus/objects/
#[no_mangle]
pub extern "C" fn alloc_tile(width: u32, height: u32) -> *mut c_void {
    let tile = TileBuffer {
        w: width,
        h: height,
        buf: BoxedSlice::with_size((width * height) as usize),
    };
    Box::into_raw(Box::new(tile)) as *mut c_void
}

unsafe fn ref_tile(tile_ptr: *mut c_void) -> &'static mut TileBuffer {
    assert!(!tile_ptr.is_null());
    &mut *(tile_ptr as *mut TileBuffer)
}

#[no_mangle]
pub extern "C" fn get_buffer(tile_ptr: *mut c_void) -> *mut u32 {
    let tile = unsafe { ref_tile(tile_ptr) };

    tile.buf.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn free_tile(tile_ptr: *mut c_void) {
    if tile_ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(tile_ptr as *mut TileBuffer);
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
fn build_palette(size: usize) -> BoxedSlice<u32> {
    let mut palette = BoxedSlice::with_size(size);
    // can shave ~200b here
    for i in 0..size {
        let m = i as f32 / size as f32;
        palette[i] = if m < 0.5 {
            let h = 200.0 / 365.0; // blue
            if m < 0.25 {
                let v = scale_value(m, 0.0, 0.25, 0.0, 1.0); // black to blue (v 0 -> 1)
                hsv_to_rgb(h, 1.0, v)
            } else {
                let s = scale_value(m, 0.25, 0.5, 1.0, 0.0); // blue to white (s 1 -> 0)
                hsv_to_rgb(h, s, 1.0)
            }
        } else {
            let h = 30.0 / 365.0; // orange
            if m < 0.75 {
                let s = scale_value(m, 0.5, 0.75, 0.0, 1.0); // white to orange (s 0 -> 1)
                hsv_to_rgb(h, s, 1.0)
            } else {
                let v = scale_value(m, 0.75, 1.0, 1.0, 0.0); // orange to black (v 1 -> 0)
                hsv_to_rgb(h, 1.0, v)
            }
        };
    }
    palette
}

fn mandel_color(i: u64, palette: &BoxedSlice<u32>) -> u32 {
    if i == 0 {
        BOTTOM
    } else {
        palette[(i % palette.len() as u64) as usize]
    }
}

// Javascript jams
#[no_mangle]
pub extern "C" fn render(
    tile_ptr: *mut c_void,
    max_iter: u32,
    center_re: f32,
    center_im: f32,
    viewport_width: f32,
) {
    let tile = unsafe { ref_tile(tile_ptr) };
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

    let step = (viewport_width / width as f32) as f64;
    let start_re = (center_re - viewport_width / 2.0) as f64;
    let start_im = (center_im - (viewport_width * (height as f32 / width as f32)) / 2.0) as f64;

    let palette = build_palette(20);

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
