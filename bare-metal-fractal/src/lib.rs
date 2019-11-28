#![feature(alloc_error_handler, lang_items, core_intrinsics)]
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
use core::intrinsics::abort;
use core::mem;
use core::ops::{Add, Deref, DerefMut};

// Debugging
use alloc::format;
use alloc::string::{String, ToString};

const DEBUG_BUFFER_SIZE: usize = 240;

static mut DEBUG_BUFFER: [u8; DEBUG_BUFFER_SIZE] = [0; DEBUG_BUFFER_SIZE];
static mut DEBUG_MSG_SIZE: usize = 0;

#[no_mangle]
pub extern "C" fn get_debug_buffer() -> *const u8 {
    unsafe { DEBUG_BUFFER.as_ptr() }
}
#[no_mangle]
pub extern "C" fn get_debug_msg_size() -> usize {
    unsafe { DEBUG_MSG_SIZE }
}

/*
https://stackoverflow.com/questions/47529643/how-to-return-a-string-or-similar-from-rust-in-webassembly
constant string buffer
log copies input string to buffer
call js
js looks up buffer address
js looks up msg len
js loads to local buffer
console logs
*/

extern "C" {
    fn js_log_msg();
}

fn log(src_full: &str) {
    unsafe {
        DEBUG_MSG_SIZE = if src_full.len() > DEBUG_BUFFER_SIZE {
            DEBUG_BUFFER_SIZE
        } else {
            src_full.len()
        };
        let src = slice::from_raw_parts(src_full.as_ptr(), DEBUG_MSG_SIZE);
        let dst = slice::from_raw_parts_mut(DEBUG_BUFFER.as_mut_ptr(), DEBUG_MSG_SIZE);
        dst.copy_from_slice(src);
        js_log_msg();
    }
}

fn log_string(src_str: String) {
    log(&src_str.to_string());
}

// Compiler calming
// implementing abort:
// https://github.com/rust-lang/rust/issues/61119
#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    unsafe {
        log("Panic!");
        abort()
    }
}

#[alloc_error_handler]
fn error_handler(_: core::alloc::Layout) -> ! {
    unsafe { abort() }
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

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

fn tween(progress: f32, s_low: f32, s_high: f32) -> f32 {
    (progress * (s_high - s_low)) + s_low
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> u32 {
    let i = (h * 6.0) as u8 as f32;
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match h {
        h if h < 0.16666666666 => rgbf(v, t, p),
        h if h < 0.33333333333 => rgbf(q, v, p),
        h if h < 0.5 => rgbf(p, v, t),
        h if h < 0.66666666666 => rgbf(p, q, v),
        h if h < 0.83333333333 => rgbf(t, p, v),
        _ => rgbf(v, p, q),
    }
}

static BOTTOM: u32 = rgb(0, 0, 0);

struct HSV(f32, f32, f32);

// golfing to do here...
fn build_palette(gradients: BoxedSlice<[&HSV; 2]>, steps_per_grad: usize) -> BoxedSlice<u32> {
    let num_gradients = gradients.len();
    let mut palette = BoxedSlice::with_size(num_gradients * steps_per_grad);
    // can shave ~200b here
    for i in 0..num_gradients {
        let color = gradients[i][0];
        let next_color = gradients[i][1];
        for step in 0..steps_per_grad {
            let progress = step as f32 / steps_per_grad as f32;
            let h = tween(progress, color.0, next_color.0);
            let s = tween(progress, color.1, next_color.1);
            let v = tween(progress, color.2, next_color.2);

            palette[i * steps_per_grad + step] = hsv_to_rgb(h, s, v);
        }
    }
    palette
}

fn mandel_color(i: u64, palette: &BoxedSlice<u32>) -> u32 {
    if i == 0 {
        BOTTOM
    } else {
        palette[((i - 1) % palette.len() as u64) as usize]
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
    let blue_h = 0.548;
    let orng_h = 0.0822;
    let blue_black = HSV(blue_h, 1.0, 0.0);
    let blue = HSV(blue_h, 1.0, 1.0);
    let blue_white = HSV(blue_h, 0.0, 1.0);
    let orange_white = HSV(orng_h, 0.0, 1.0);
    let orange = HSV(orng_h, 1.0, 1.0);
    let orange_black = HSV(orng_h, 1.0, 0.0);

    let gradients = BoxedSlice(Box::from([
        [&blue_black, &blue],
        [&blue, &blue_white],
        [&orange_white, &orange],
        [&orange, &orange_black],
    ]));

    let palette = build_palette(gradients, 4);

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
