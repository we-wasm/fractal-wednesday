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

#[derive(Debug, Clone)]
struct SampleSpace {
    tile: TileSpace,
    coord: Point<f32>, // u,v 0-1
}

impl SampleSpace {
    fn from_complex(c: &ComplexSpace, z: usize) -> SampleSpace {
        let zoom_power = pow(2, z) as f64;
        let tile_x = zoom_power * c.re;
        let tile_y = zoom_power * c.im;

        let sample_x = (tile_x % 1.0) as f32;
        let sample_x = if sample_x < 0.0 {
            1.0 + sample_x
        } else {
            sample_x
        };
        let sample_y = (tile_y % 1.0) as f32;
        let sample_y = if sample_y < 0.0 {
            1.0 + sample_y
        } else {
            sample_y
        };

        SampleSpace {
            tile: TileSpace {
                index: Point {
                    x: tile_x.floor() as i64,
                    y: tile_y.floor() as i64,
                },
                zoom: z,
            },
            coord: Point {
                x: sample_x,
                y: sample_y,
            },
        }
    }
}

#[derive(Debug, Clone)]
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
    fn sample(&self, data: &Tile, coord: &Point<f32>) -> u64;
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
    fn sample(&mut self, location: ComplexSpace, zoom: usize) -> u64;
}
type TileHash = String;

struct TileStorage {
    generator: Box<dyn Generator>,
    storage: HashMap<TileHash, Tile>,
}

// Renderer
trait PixelRenderer {
    fn render(&mut self, viewport: &ViewportConfig) -> Vec<RGB8>;
}

struct RenderConfig {
    manager: Box<dyn TileManager>,
    palette: Vec<RGB8>,
    bottom: RGB8,
    size: Point<usize>,
    tile_width: f32,
}

struct ViewportConfig {
    center: ComplexSpace,
    zoom: f64, // big decimal
}

impl Generator for GeneratorConfig {
    fn generate(&self, tile: &TileSpace) -> Tile {
        println!("Creating tile {:?}", tile);
        let z: usize = tile.zoom;
        let start = ComplexSpace::from(tile);
        let end = ComplexSpace::from(&TileSpace {
            index: Point {
                x: tile.index.x + 1,
                y: tile.index.y + 1,
            },
            zoom: z,
        });

        let step_x = (end.re - start.re) / self.size.x as f64;
        let step_y = (end.im - start.im) / self.size.y as f64;

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

    fn sample(&self, tile: &Tile, coord: &Point<f32>) -> u64 {
        let x = (self.size.x as f32 * coord.x) as usize;
        let y = (self.size.y as f32 * coord.y) as usize;

        return tile.data[((y * self.size.y) + x) as usize];
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

    if iter == max_iter {
        0
    } else {
        iter
    }
}

impl TileManager for TileStorage {
    fn sample(&mut self, location: ComplexSpace, zoom: usize) -> u64 {
        let sample = SampleSpace::from_complex(&location, zoom);

        let hash = self.generator.hash(&sample.tile);

        if !self.storage.contains_key(&hash) {
            let gen = self.generator.generate(&sample.tile);
            self.storage.insert(String::clone(&hash), gen);
        }
        let tile = self.storage.get(&hash).unwrap();

        self.generator.sample(tile, &sample.coord)
    }
}

impl PixelRenderer for RenderConfig {
    fn render(&mut self, viewport: &ViewportConfig) -> Vec<RGB8> {
        // How wide is the viewport in complex space
        let complex_w = self.tile_width as f64 / (2.0 as f64).powf(viewport.zoom);
        let complex_h = self.size.y as f64 / self.size.x as f64 * complex_w;

        let step = complex_w / self.size.x as f64;

        let start_x = viewport.center.re - complex_w / 2.0;
        let start_y = viewport.center.im - complex_h / 2.0;

        let mut data: Vec<RGB8> =
            vec![RGB8 { r: 0, g: 0, b: 0 }; (self.size.x * self.size.y) as usize];

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let iter = self.manager.sample(
                    ComplexSpace(Complex64 {
                        re: start_x + x as f64 * step,
                        im: start_y + y as f64 * step,
                    }),
                    viewport.zoom.floor() as usize,
                );

                data[((y * self.size.y) + x) as usize] = if iter == 0 {
                    self.bottom
                } else {
                    self.palette[(iter % self.palette.len() as u64) as usize]
                };
            }
        }

        data
    }
}

fn main() {
    let generator = GeneratorConfig {
        max_iter: 5000,
        size: Point { x: 60, y: 60 },
    };

    let manager = TileStorage {
        generator: Box::new(generator),
        storage: HashMap::new(),
    };

    let mut renderer = RenderConfig {
        manager: Box::new(manager),
        palette: vec![
            RGB8 {
                r: 255,
                g: 255,
                b: 255,
            },
            RGB { r: 255, g: 0, b: 0 },
        ],
        bottom: RGB { r: 0, g: 0, b: 0 },
        size: Point { x: 300, y: 300 },
        tile_width: 3.0,
    };
    let viewport = ViewportConfig {
        center: ComplexSpace(Complex64 { re: -0.5, im: 0.0 }),
        zoom: 0.5,
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
