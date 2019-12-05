extern crate wasm_bindgen;
extern crate wee_alloc;

use std::ops::Add;
use std::slice;

use web_sys::{CanvasRenderingContext2d, ImageData};

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(debug_assertions)]
#[macro_use]
mod debug {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        // Use `js_namespace` here to bind `console.log(..)` instead of just
        // `log(..)`
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);
    }

    macro_rules! dbg {
        // Note that this is using the `log` function imported above during
        // `bare_bones`
        ($($t:tt)*) => (debug::log(&format_args!($($t)*).to_string()))
    }
}
#[cfg(not(debug_assertions))]
#[macro_use]
mod debug {
    macro_rules! dbg {
        ($($arg:tt)*) => {{}};
    }
}

pub struct TileBuffer {
    w: usize,
    h: usize,
    buf: Vec<RGB>,
}

impl TileBuffer {
    fn with_size(width: usize, height: usize) -> Self {
        TileBuffer {
            w: width,
            h: height,
            buf: vec![RGB::rgb(0, 0, 0); (width * height) as usize],
        }
    }

    fn get_mut_buf(&mut self) -> &mut [u8] {
        // the buf is exactly the same as the expected array of bytes
        // just represented in chunks of 4
        unsafe {
            let u8_ptr = &mut *(self.buf.as_mut_ptr() as *mut RGB as *mut u8);
            slice::from_raw_parts_mut(u8_ptr, self.w * self.h * 4)
        }
    }
}

// Complex coordination
// https://rustwasm.github.io/wasm-bindgen/examples/julia.html
#[derive(Clone, Copy, Debug)]
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
#[repr(C)]
struct RGB {
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
    fn rgb(r: u8, g: u8, b: u8) -> Self {
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

fn build_palette(gradients: &Vec<[&RGB; 2]>, steps_per_grad: usize) -> Vec<RGB> {
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

fn mandel_color(i: u64, palette: &Vec<RGB>) -> RGB {
    if i == 0 {
        BOTTOM
    } else {
        // This is on the hot loop, can len be removed?
        palette[(i % palette.len() as u64) as usize]
    }
}

#[derive(Debug)]
pub struct Viewport {
    center: Complex,
    width: f64,
    max_iter: u64,
}

impl Viewport {
    fn translate(&mut self, u: f32, v: f32) {
        self.center = Complex {
            re: self.center.re + (self.width * u as f64),
            im: self.center.im + (self.width * v as f64),
        }
    }
}

pub enum MouseState {
    Up,
    Down(f32, f32),
}
pub struct UiState {
    viewport: Viewport,
    mouse: MouseState,
    tile: TileBuffer,
    ctx: CanvasRenderingContext2d,
}

pub enum UiAction {
    MouseUp,
    MouseDown(f32, f32),
    MouseMove(f32, f32),
    Resize(usize, usize),
}

impl UiState {
    pub fn init(ctx: CanvasRenderingContext2d, width: usize, height: usize) -> UiState {
        UiState {
            tile: TileBuffer::with_size(width, height),
            viewport: Viewport {
                center: Complex { re: -0.5, im: 0.0 },
                width: 3.0,
                max_iter: 100,
            },
            ctx,
            mouse: MouseState::Up,
        }
    }

    pub fn handle(&mut self, a: UiAction) {
        match a {
            UiAction::MouseUp => self.mouse = MouseState::Up,
            UiAction::MouseDown(u, v) => self.mouse = MouseState::Down(u, v),
            UiAction::MouseMove(u, v) => {
                // TODO: mutate viewport based on a.mouse - s.mouse
                // self.viewport = manipulate viewport
                if let MouseState::Down(prev_u, prev_v) = self.mouse {
                    let scaled_v = (prev_v - v) * (self.tile.h as f32 / self.tile.w as f32);
                    self.viewport.translate(prev_u - u, scaled_v);
                    self.mouse = MouseState::Down(u, v);
                    self.render();
                }
            }
            UiAction::Resize(w, h) => {
                self.tile = TileBuffer::with_size(w, h);
                self.render();
            }
        }
    }

    fn render(&mut self) {
        let tile = &mut self.tile;
        let width = tile.w;
        let height = tile.h;
        let viewport = &self.viewport;
        let viewport_width = viewport.width;
        let center_re = viewport.center.re;
        let center_im = viewport.center.im;
        let max_iter = viewport.max_iter;

        let step = viewport_width / width as f64;
        let start_re = (center_re - viewport_width / 2.0) as f64;
        let start_im = (center_im - (viewport_width * (height as f64 / width as f64)) / 2.0) as f64;
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
                tile.buf[(y * width + x) as usize] = mandel_color(
                    mandel_iter(
                        max_iter as u64,
                        Complex {
                            re: start_re + ((x as f64) * step),
                            im: start_im + ((y as f64) * step),
                        },
                    ),
                    &palette,
                );
            }
        }

        let data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(tile.get_mut_buf()),
            width as u32,
            height as u32,
        )
        .unwrap();
        self.ctx.put_image_data(&data, 0.0, 0.0).unwrap();
    }
}

impl Drop for UiState {
    fn drop(&mut self) {
        dbg!("Dropping ui state");
    }
}

// Javascript jams
// let the jank begin!

static mut STATES: Option<Vec<UiState>> = None;
type StateId = usize;
fn get_ui_states() -> &'static mut Vec<UiState> {
    unsafe {
        if let None = STATES {
            STATES = Some(vec![]);
        }
        STATES.as_mut().unwrap()
    }
}

fn get_state(id: StateId) -> &'static mut UiState {
    let ui_states = get_ui_states();
    ui_states.get_mut(id).expect("Use of uninitialized UiState")
}

#[wasm_bindgen]
pub fn mount(ctx: CanvasRenderingContext2d, width: usize, height: usize) -> StateId {
    let ui_states = get_ui_states();
    let mut ui_state = UiState::init(ctx, width, height);
    ui_state.render();
    ui_states.push(ui_state);
    return ui_states.len() - 1;
}

#[wasm_bindgen]
pub fn resize(s: StateId, width: usize, height: usize) {
    get_state(s).handle(UiAction::Resize(width, height));
}

#[wasm_bindgen]
pub fn mouse_down(s: StateId, u: f32, v: f32) {
    get_state(s).handle(UiAction::MouseDown(u, v));
}

#[wasm_bindgen]
pub fn mouse_up(s: StateId) {
    get_state(s).handle(UiAction::MouseUp);
}

#[wasm_bindgen]
pub fn mouse_move(s: StateId, u: f32, v: f32) {
    get_state(s).handle(UiAction::MouseMove(u, v));
}
