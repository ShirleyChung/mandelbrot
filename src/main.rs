use num::Complex;
use std::cmp::max;
use std::str::FromStr;
use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex {re, im}),
        None => None
    }
}

#[allow(dead_code)]
fn complex_square_add_loop(c: Complex<f64>) {
    let mut z = Complex {re: 0.0, im: 0.0};
    loop {
        z = z * z + c;
    }
}

fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex {re: 0.0, im: 0.0};
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
    None
}

fn pixel_to_point (bounds: (usize, usize), pixels: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>) -> Complex<f64> {
    let (width, height) = (lower_right.re - upper_left.re,
                            upper_left.im - lower_right.im);
    Complex { re: (upper_left.re + pixels.0 as f64 * width / bounds.0 as f64), im: (upper_left.im - pixels.1 as f64 * height / bounds.1 as f64) }
}

fn render(pixels: &mut [u8], bounds: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>) {
    for row in 0..bounds.1 {
        for col in 0..bounds.0 {
            let point = pixel_to_point(bounds, (col, row), upper_left, lower_right);
            pixels[row * bounds.0 + col] = 
            match escape_time(point, 255) {
                Some(c) => 255 - c as u8,
                None => 0
            }

        }
    }
}

fn wirte_image(filename: &str, data: &[u8], bounds: (usize, usize)) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);
    encoder.encode(data, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut filename = "mandelbrot.png";
    let mut bounds = (1000, 750);
    let mut ul = Complex {re: -1.2, im:0.35};
    let mut lr = Complex {re: -1.0, im:0.20};
    let mut threads = 64; // 把圖切成8塊分別畫
    if args.len() > 1 {
        filename = &args[1];
        if args.len() > 2 {
            bounds = parse_pair(&args[2], 'x')
            .expect("error on parse dimension");
            if args.len() > 3 {
                ul = parse_complex(&args[3]).expect("error on parse upper left");
                if args.len() > 4 {
                    lr = parse_complex(&args[4]).expect("error on parse lower right");
                    if args.len() > 5 {
                        threads = usize::from_str(&args[5]).expect("error specify thread nums");
                        threads = max(1, threads);
                    }
                }
            }
        }
    }
    let mut pixels = vec![0; bounds.0 as usize * bounds.1 as usize];
    println!("create PNG file: {} with bounds: {:?}, upper left: {} lower right: {}, threads: {}", filename, bounds, ul, lr, threads);

    let rows_per_band = bounds.1 / threads + 1; // 至少1列
    let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();
    crossbeam::scope(|spawner| {
        for (i, band) in bands.into_iter().enumerate() {
            let top = rows_per_band * i;
            let height = band.len() / bounds.0;
            let band_bound = (bounds.0, height);
            let band_upper_left = pixel_to_point(bounds, (0, top), ul, lr);
            let band_lower_right = pixel_to_point(bounds, (bounds.0, top + height), ul, lr);
            spawner.spawn(move |_| {
                render(band, band_bound, band_upper_left, band_lower_right);
            });
        }
    }).unwrap();

    wirte_image(filename, &pixels, bounds)
    .expect("error on write PNG file");
}
