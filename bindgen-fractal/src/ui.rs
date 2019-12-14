// pixel buffer <-> events
use crate::render::{build_palette, mandel_color, mandel_iter, Complex, RGB};
use crate::viewport::{TileBuffer, Viewport};

pub enum MouseState {
    Up,
    Down(f32, f32),
}
pub struct UiState {
    viewport: Viewport,
    mouse: MouseState,
    max_iter: u64,
    tile: Option<TileBuffer<RGB>>,
}

pub enum UiAction {
    MouseUp,
    MouseDown(f32, f32),
    MouseMove(f32, f32),
    MouseZoom(f32, f32, f32),
}

impl UiState {
    pub fn init() -> UiState {
        UiState {
            tile: None,
            viewport: Viewport {
                center: Complex { re: -0.5, im: 0.0 },
                width: 3.0,
            },
            max_iter: 100,
            mouse: MouseState::Up,
        }
    }

    pub fn handle(&mut self, a: UiAction) {
        use UiAction::*;
        match a {
            MouseUp => self.mouse = MouseState::Up,
            MouseDown(u, v) => self.mouse = MouseState::Down(u, v),
            MouseMove(u, v) => {
                // TODO: mutate viewport based on a.mouse - s.mouse
                // self.viewport = manipulate viewport
                if let MouseState::Down(prev_u, prev_v) = self.mouse {
                    if let Some(tile) = &self.tile {
                        let scaled_v = (prev_v - v) * tile.aspect_ratio();
                        self.viewport.translate(prev_u - u, scaled_v);
                        self.mouse = MouseState::Down(u, v);
                    }
                }
            }
            MouseZoom(z, u, v) => {
                if let Some(tile) = &self.tile {
                    let scaled_v = ((v - 0.5) * tile.aspect_ratio()) + 0.5;
                    self.viewport.zoom(z, u, scaled_v);
                }
            }
        }
    }

    pub fn render(&mut self, w: usize, h: usize) -> &mut [u8] {
        self.tile = Some(match self.tile.take() {
            Some(t) => t.resize(w, h, RGB::rgb(0, 0, 0)),
            None => TileBuffer::with_size(w, h, RGB::rgb(0, 0, 0)),
        });
        let tile = self.tile.as_mut().unwrap();

        let viewport = &self.viewport;

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

        let max_iter = self.max_iter;

        tile.apply(viewport.center, viewport.width, |c| {
            mandel_color(mandel_iter(max_iter, c), &palette)
        });

        tile.get_mut_buf()
    }
}

impl Drop for UiState {
    fn drop(&mut self) {
        dbg!("Dropping ui state");
    }
}
