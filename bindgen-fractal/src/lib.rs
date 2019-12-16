extern crate wasm_bindgen;
extern crate wee_alloc;

use web_sys::{CanvasRenderingContext2d, ImageData, Storage};

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
        ($($t:tt)*) => (crate::debug::log(&format_args!($($t)*).to_string()))
    }
}
#[cfg(not(debug_assertions))]
#[macro_use]
mod debug {
    macro_rules! dbg {
        ($($arg:tt)*) => {{}};
    }
}

// Javascript jams
// let the jank begin!
mod render;
mod ui;
mod viewport;

use ui::{UiAction, UiState};

static mut STATES: Option<Vec<UiState>> = None;
type StateId = usize;
fn get_ui_states() -> &'static mut Vec<UiState> {
    // Need to maintain rust ownership for memory management
    // Passing static ints over to js, letting wasm runtime manage realloc
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
pub fn new_fractal() -> StateId {
    let ui_states = get_ui_states();
    let ui_state = UiState::init();
    ui_states.push(ui_state);
    return ui_states.len() - 1;
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

#[wasm_bindgen]
pub fn zoom(s: StateId, z: f32, u: f32, v: f32) {
    get_state(s).handle(UiAction::MouseZoom(z, u, v));
}

#[wasm_bindgen]
pub fn render(s: StateId, ctx: CanvasRenderingContext2d, w: u32, h: u32) {
    let ui_state = get_state(s);
    let buf = ui_state.render(w as usize, h as usize);
    let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(buf), w, h).unwrap();
    ctx.put_image_data(&data, 0.0, 0.0).unwrap();
}
