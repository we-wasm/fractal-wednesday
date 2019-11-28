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
use core::ops::{Add, Deref, DerefMut, Index, IndexMut};

// Debugging
#[cfg(debug_assertions)]
#[macro_use]
mod debug {
    use alloc::slice;
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

    extern "C" {
        fn js_log_msg();
    }

    pub fn log_string(src_str: String) {
        let src_full = &src_str.to_string();
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

    macro_rules! dbg {
        ($($arg:tt)*) => (debug::log_string(alloc::format!($($arg)*)))
    }
}
#[cfg(not(debug_assertions))]
#[macro_use]
mod debug {
    macro_rules! dbg {
        ($($arg:tt)*) => {{}};
    }
}

// Compiler calming
// implementing abort:
// https://github.com/rust-lang/rust/issues/61119
#[panic_handler]
fn handle_panic(panic_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if cfg!(debug_assertions) {
            match panic_info.payload().downcast_ref::<&str>() {
                Some(v) => dbg!("Panic: {}", v),
                None => dbg!("Unknown Panic"),
            };
        }
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
    buf: BoxedSlice<RGB>,
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
            let s = slice::from_raw_parts_mut(ptr, size);
            BoxedSlice(Box::from_raw(s))
        }
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
impl<T> Index<usize> for BoxedSlice<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        unsafe { self.get_unchecked(i) }
    }
}
impl<T> IndexMut<usize> for BoxedSlice<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        unsafe { self.get_unchecked_mut(i) }
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
pub extern "C" fn get_buffer(tile_ptr: *mut c_void) -> *mut c_void {
    let tile = unsafe { ref_tile(tile_ptr) };

    tile.buf.as_mut_ptr() as *mut c_void
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

// With this byte order javascript can copy it straight into canvas
#[repr(C)]
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
fn build_palette(gradients: &BoxedSlice<[&RGB; 2]>, steps_per_grad: usize) -> BoxedSlice<RGB> {
    let num_gradients = gradients.len();
    let mut palette = BoxedSlice::with_size(num_gradients * steps_per_grad);
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

fn mandel_color(i: u64, palette: &BoxedSlice<RGB>) -> RGB {
    if i == 0 {
        BOTTOM
    } else {
        // This is on the hot loop, can len be removed?
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
    let blue = RGB::rgb(0, 183, 255);
    let orange = RGB::rgb(255, 128, 0);
    let black = RGB::rgb(0, 0, 0);
    let white = RGB::rgb(255, 255, 255);

    let gradients = BoxedSlice(Box::from([
        [&black, &blue],
        [&blue, &white],
        [&white, &orange],
        [&orange, &black],
    ]));

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
