/*!
Copyright 2025 Lucas Walter

Show animated cubic bezier, move the handles and end points around randomly
*/

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use noise::{NoiseFn, OpenSimplex};
use stroke::f64::{CubicBezier, Point, PointN};

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
    let handle_scale = wd * 0.7;
    let mut count = 0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // end point
        let fr = count as f64 / 2000.0;
        let noise_pt = [fr, 0.0];
        let x0 = x0_simplex.get(noise_pt) * wd + wd / 2.0;
        let noise_pt = [0.0, fr];
        let y0 = y0_simplex.get(noise_pt) * ht + ht / 2.0;
        let c0 = PointN::new([x0, y0]);

        // handle
        let fr = count as f64 / 250.0;
        let noise_pt = [fr, 0.0];
        let x1 = x1_simplex.get(noise_pt) * handle_scale;
        let noise_pt = [0.0, fr];
        let y1 = y1_simplex.get(noise_pt) * handle_scale;
        let h0 = PointN::new([x0 + x1, y0 + y1]);

        // end point
        let fr = count as f64 / 2000.0;
        let noise_pt = [fr, 0.0];
        let x3 = x3_simplex.get(noise_pt) * wd + wd / 2.0;
        let noise_pt = [0.0, fr];
        let y3 = y3_simplex.get(noise_pt) * ht + ht / 2.0;
        let c3 = PointN::new([x3, y3]);

        // handle
        let fr = count as f64 / 250.0;
        let noise_pt = [fr, 0.0];
        let x2 = x2_simplex.get(noise_pt) * handle_scale;
        let noise_pt = [0.0, fr];
        let y2 = y2_simplex.get(noise_pt) * handle_scale;
        let h1 = PointN::new([x3 + x2, y3 + y2]);

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

            let pix_ind = y as usize * width + x as usize;
            if pix_ind >= 0 && pix_ind < (width * height) {
                let a = 255;
                let r = 255;
                let g = 200;
                let b = 180;
                argb[pix_ind] = a << 24 | r << 16 | g << 8 | b;
            }
        }

        for pt in [h0, h1] {
            let pix_ind = pt.axis(0) as usize * width + pt.axis(1) as usize;
            if pix_ind >= 0 && pix_ind < (width * height) {
                let a = 255;
                let r = 25;
                let g = 250;
                let b = 130;
                argb[pix_ind] = a << 24 | r << 16 | g << 8 | b;
            }
        }

        window.update_with_buffer(&argb, width, height).unwrap();

        count += 1;
    }
}
