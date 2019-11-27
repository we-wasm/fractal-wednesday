#![feature(alloc_error_handler, lang_items)]
#![no_std]

extern crate alloc;
extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloc::boxed::Box;
use alloc::vec::Vec;

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
    fn js_log(x: u32);
}

#[repr(C)]
pub struct TileBuffer {
    w: u32,
    h: u32,
    buf: Vec<u32>,
}

// http://jakegoulding.com/rust-ffi-omnibus/objects/
#[no_mangle]
pub extern "C" fn make_viewport(width: u32, height: u32) -> *mut TileBuffer {
    let viewport = TileBuffer {
        w: width,
        h: height,
        buf: Vec::with_capacity((width * height) as usize),
    };
    Box::into_raw(Box::new(viewport))
}

#[no_mangle]
pub extern "C" fn get_buffer(viewport_ptr: *mut TileBuffer) -> *mut u32 {
    let viewport = unsafe {
        assert!(!viewport_ptr.is_null());
        &mut *viewport_ptr
    };

    viewport.buf.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn free_viewport(viewport: *mut TileBuffer) {
    if viewport.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(viewport);
    }
}

#[no_mangle]
pub extern "C" fn render(viewport_ptr: *mut TileBuffer) {
    let viewport = unsafe {
        assert!(!viewport_ptr.is_null());
        &mut *viewport_ptr
    };
    render_frame_safe(viewport)
}

// We split this out so that we can escape 'unsafe' as quickly
// as possible.
fn render_frame_safe(viewport: &mut TileBuffer) {
    let width = viewport.w;
    let height = viewport.h;
    let buf = viewport.buf.as_mut_ptr();
    for y in 0..height {
        for x in 0..width {
            unsafe {
                let p = buf.offset((y * width + x) as isize);
                *p = (x ^ y) as u32 | 0xFF_00_00_00;
            }
        }
    }
}
