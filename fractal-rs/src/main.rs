extern crate num;

use num::complex::Complex64;
use num::pow::pow;

struct Point<T> {
    x: T,
    y: T,
}

struct Viewport {
    center: Point<f64>,
    size: Point<u32>,
}

enum FractalAlgorithm {
    NaiveMandlebrot = 1,
}

struct TileConfig {
    index: Point<i64>,
    zoom: usize,
    size: Point<u32>,
    max_iter: u64,
    algorithm: FractalAlgorithm,
}

struct Tile {
    config: TileConfig,
    data: Vec<u64>,
}

struct TileGeneration {
    start: Point<f64>,
    step: Point<f64>,
}

fn transform_index(p: &Point<i64>, z: usize) -> Point<f64> {
    let p: Point<f64> = Point {
        x: p.x as f64,
        y: p.y as f64,
    };

    return Point {
        x: p.x / pow(2.0, z),
        y: p.y / pow(2.0, z),
    };
}

fn calc_gen_info(config: &TileConfig) -> TileGeneration {
    let z: usize = config.zoom;
    let start: Point<f64> = transform_index(&config.index, z);
    let end: Point<f64> = transform_index(
        &Point {
            x: config.index.x + 1,
            y: config.index.y + 1,
        },
        z,
    );

    return TileGeneration {
        step: Point {
            x: (end.x - start.x) / config.size.x as f64,
            y: (end.y - start.y) / config.size.y as f64,
        },
        start: start,
    };
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

// The serial version of the mandelbrot set calculation.
fn serial(config: TileConfig) -> Tile {
    let mut data: Vec<u64> = vec![0; (config.size.x * config.size.y) as usize];

    let gen = calc_gen_info(&config);

    for y in 0..config.size.y {
        for x in 0..config.size.x {
            data[((y * config.size.y) + x) as usize] = mandel_iter(
                config.max_iter,
                Complex64 {
                    re: gen.start.x + ((x as f64) * gen.step.x),
                    im: gen.start.y + ((y as f64) * gen.step.y),
                },
            );
        }
    }

    Tile { config, data }
}

fn main() {
    println!("Hello, world!");
}
