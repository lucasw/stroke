/*!
Copyright 2025 Lucas Walter

Show animated cubic bezier, move the handles and end points around randomly
*/

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use noise::{NoiseFn, OpenSimplex};
use stroke::f64::{CubicBezier, Point, PointN};

fn put_pixel(argb: &mut [u32], x: f64, y: f64, width: usize, height: usize, r: u8, g: u8, b: u8) {
    if x < 0.0 || y < 0.0 {
        return;
    }
    let x = (x + 0.5) as usize;
    let y = (y + 0.5) as usize;
    if x >= width || y >= height {
        return;
    }
    let pix_ind = y as usize * width + x as usize;
    let a = 255;
    argb[pix_ind] = a << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
}

fn get_line_pixels(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<(f64, f64)> {
    let dy = y1 - y0;
    let dx = x1 - x0;

    let mut pixel_points = Vec::new();

    let inc = 1;
    let incf = inc as f64;

    let (num_steps, x_step, y_step) = {
        if dx.abs() > dy.abs() {
            let slope = dy / dx.abs();
            let num_steps = (dx.abs() as i32).max(1);
            let x_step = incf * dx.signum();
            let y_step = slope * incf;
            (num_steps, x_step, y_step)
        } else {
            let slope = dx / dy.abs();
            let num_steps = (dy.abs() as i32).max(1);
            let x_step = slope * incf;
            let y_step = incf * dy.signum();
            (num_steps, x_step, y_step)
        }
    };

    let mut x = x0;
    let mut y = y0;
    for _ in (0..num_steps).step_by(inc) {
        pixel_points.push((x, y));
        x += x_step;
        y += y_step;
    }

    // println!("{x0:.2} {y0:.2} to {x1:.2} {y1:.2}");
    // println!("{pixel_points:?}");
    pixel_points
}

fn main() {
    let width = 320;
    let height = 180;

    let mut window = Window::new(
        "cubic_bezier",
        width,
        height,
        WindowOptions {
            resize: false,
            scale: Scale::X4,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("unable to create window");

    let mut argb = vec![0u32; width * height];

    let x0_simplex = OpenSimplex::new(10);
    let y0_simplex = OpenSimplex::new(11);
    let x1_simplex = OpenSimplex::new(12);
    let y1_simplex = OpenSimplex::new(13);

    let x2_simplex = OpenSimplex::new(20);
    let y2_simplex = OpenSimplex::new(21);
    let x3_simplex = OpenSimplex::new(22);
    let y3_simplex = OpenSimplex::new(23);

    let wd = width as f64;
    let ht = height as f64;
    let handle_scale = wd * 0.6;
    let scale = 0.8;
    let mut count = 0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // end point
        let fr = count as f64 / 2000.0;
        let noise_pt = [fr, 0.0];
        let x0 = x0_simplex.get(noise_pt) * wd * scale + wd / 2.0;
        let noise_pt = [0.0, fr];
        let y0 = y0_simplex.get(noise_pt) * ht * scale + ht / 2.0;
        let c0 = PointN::new([x0, y0]);

        // handle
        let fr = count as f64 / 250.0;
        let noise_pt = [fr, 0.0];
        let x1 = x0 + x1_simplex.get(noise_pt) * handle_scale;
        let noise_pt = [0.0, fr];
        let y1 = y0 + y1_simplex.get(noise_pt) * handle_scale;
        let h0 = PointN::new([x1, y1]);

        // end point
        let fr = count as f64 / 2000.0;
        let noise_pt = [fr, 0.0];
        let x3 = x3_simplex.get(noise_pt) * wd * scale + wd / 2.0;
        let noise_pt = [0.0, fr];
        let y3 = y3_simplex.get(noise_pt) * ht * scale + ht / 2.0;
        let c3 = PointN::new([x3, y3]);

        // handle
        let fr = count as f64 / 250.0;
        let noise_pt = [fr, 0.0];
        let x2 = x3 + x2_simplex.get(noise_pt) * handle_scale;
        let noise_pt = [0.0, fr];
        let y2 = y3 + y2_simplex.get(noise_pt) * handle_scale;
        let h1 = PointN::new([x2, y2]);

        let bezier = CubicBezier::<PointN<2>, 2>::new(c0, h0, h1, c3);

        for elem in argb.iter_mut() {
            *elem = 0u32;
        }

        let nsteps = 512;
        for ind in 0..nsteps {
            let s = ind as f64 / nsteps as f64;
            let p = bezier.eval(s);
            let x = p.axis(0);
            let y = p.axis(1);

            if ind == 0 || ind == (nsteps - 1) {
                // println!("{ind} {x:.2} {y:.2}");
            }

            put_pixel(&mut argb, x, y, width, height, 255, 255, 128);
        }

        let handle_points0 = get_line_pixels(x0, y0, x1, y1);
        let handle_points1 = get_line_pixels(x3, y3, x2, y2);
        for handle_points in [handle_points0, handle_points1] {
            for pt in handle_points {
                put_pixel(&mut argb, pt.0, pt.1, width, height, 50, 250, 90);
            }
        }

        window.update_with_buffer(&argb, width, height).unwrap();

        count += 1;
    }
}
