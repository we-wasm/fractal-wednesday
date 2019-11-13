extern crate num;
extern crate rgb;

use num::complex::Complex64;
use num::pow::pow;
use rgb::*;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Clone)]
struct Point<T> {
    x: T,
    y: T,
}

#[derive(Debug, Clone)]
struct TileSpace {
    // unique data per tile
    index: Point<i64>, // big int
    zoom: usize,       // big int
}

struct ComplexSpace(Complex64); // big decimal
impl ComplexSpace {
    fn from(t: &TileSpace) -> Self {
        return ComplexSpace(Complex64 {
            re: t.index.x as f64 / pow(2.0, t.zoom),
            im: t.index.y as f64 / pow(2.0, t.zoom),
        });
    }
}
impl Deref for ComplexSpace {
    type Target = Complex64;

    fn deref(&self) -> &Complex64 {
        &self.0
    }
}

// Generator
trait Generator {
    fn generate(&self, tile: &TileSpace) -> Tile;
    fn hash(&self, tile: &TileSpace) -> TileHash;
}

struct GeneratorConfig {
    size: Point<usize>,
    max_iter: u64, // big integer?
}

struct Tile {
    data: Vec<u64>,
}

// Tile Manager
trait TileManager {
    fn sample(&self, location: ComplexSpace, zoom: f64) -> u64;
}
type TileHash = String;

struct TileStorage {
    generator: Box<dyn Generator>,
    storage: HashMap<TileHash, Tile>,
}

// Renderer
trait PixelRenderer {
    fn render(&self, viewport: &ViewportConfig) -> Vec<RGB8>;
}

struct RenderConfig {
    manager: Box<dyn TileManager>,
    palette: Vec<RGB8>,
    bottom: RGB8,
    size: Point<usize>,
}

struct ViewportConfig {
    center: ComplexSpace,
    zoom: f64, // big decimal
}

impl Generator for GeneratorConfig {
    fn generate(&self, tile: &TileSpace) -> Tile {
        let z: usize = tile.zoom;
        let start = ComplexSpace::from(tile);
        let end = ComplexSpace::from(&TileSpace {
            index: Point {
                x: tile.index.x + 1,
                y: tile.index.y + 1,
            },
            zoom: z,
        });

        let step_x: f64 = (end.re - start.re) / self.size.x as f64;
        let step_y: f64 = (end.im - start.im) / self.size.y as f64;

        let mut data: Vec<u64> = vec![0; (self.size.x * self.size.y) as usize];

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                data[((y * self.size.y) + x) as usize] = mandel_iter(
                    self.max_iter,
                    Complex64 {
                        re: start.re + ((x as f64) * step_x),
                        im: start.im + ((y as f64) * step_y),
                    },
                );
            }
        }

        Tile { data }
    }

    fn hash(&self, tile: &TileSpace) -> TileHash {
        return format!(
            "{}x{}-{}-x{}y{}z{}",
            self.size.x, self.size.y, self.max_iter, tile.index.x, tile.index.y, tile.zoom
        );
    }
}

// via https://github.com/willi-kappler/mandel-rust/blob/master/mandel_method/src/lib.rs
// The inner iteration loop of the mandelbrot calculation
// See https://en.wikipedia.org/wiki/Mandelbrot_set
pub fn mandel_iter(max_iter: u64, c: Complex64) -> u64 {
    let mut z: Complex64 = c;

    let mut iter = 0;

    while (z.norm_sqr() <= 4.0) && (iter < max_iter) {
        z = c + (z * z);
        iter = iter + 1;
    }

    iter
}

impl TileManager for TileStorage {
    fn sample(&self, location: ComplexSpace, zoom: f64) -> u64 {
        //
    }
}

impl PixelRenderer for RenderConfig {
    fn render(&self, viewport: &ViewportConfig) -> Vec<RGB8> {}
}

fn main() {
    let generator = GeneratorConfig {
        max_iter: 5000,
        size: Point { x: 256, y: 256 },
    };

    let manager = TileStorage {
        generator: Box::new(generator),
        storage: HashMap::new(),
    };

    let renderer = RenderConfig {
        manager: Box::new(manager),
        palette: vec![
            RGB8 {
                r: 255,
                g: 255,
                b: 255,
            },
            RGB { r: 0, g: 0, b: 0 },
        ],
        bottom: RGB { r: 255, g: 0, b: 0 },
        size: Point { x: 512, y: 512 },
    };
    let viewport = ViewportConfig {
        center: ComplexSpace(Complex64 { re: 0.0, im: 0.0 }),
        zoom: 0.0,
    };

    let pixels = renderer.render(&viewport);

    if let Err(e) = lodepng::encode_file(
        "mandel.png",
        &pixels,
        renderer.size.x,
        renderer.size.y,
        lodepng::ColorType::RGB,
        8,
    ) {
        panic!("failed to write png: {:?}", e);
    }
}
// cargo run  120.34s user 0.54s system 98% cpu 2:02.46 total
