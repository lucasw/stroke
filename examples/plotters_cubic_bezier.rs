extern crate plotters;
use plotters::prelude::*;

extern crate stroke;
use stroke::CubicBezier;
use stroke::Point;
use stroke::PointN;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // control points for the cubic bezier curve
    let cpoints = vec![
        (0f64, 1.77f64),
        (1.1f64, -1f64),
        (4.3f64, 3f64),
        (3.2f64, -4f64),
    ];

    let bezier = CubicBezier::new(
        PointN::new([cpoints[0].0, cpoints[0].1]),
        PointN::new([cpoints[1].0, cpoints[1].1]),
        PointN::new([cpoints[2].0, cpoints[2].1]),
        PointN::new([cpoints[3].0, cpoints[3].1]),
    );

    let bounds = bezier.bounding_box();
    let xmin = bounds[0].0;
    let xmax = bounds[0].1;
    let dx = xmax - xmin;
    let ymin = bounds[1].0;
    let ymax = bounds[1].1;
    let dy = ymax - ymin;
    let dmax = dx.max(dy);

    // render the paths of the curve to desired accuracy
    let nsteps: usize = 1000;
    let mut bezier_graph: Vec<(f64, f64)> = Vec::with_capacity(nsteps);
    let mut bezier_graph_reg: Vec<(f64, f64)> = Vec::with_capacity(nsteps);
    for t in 0..nsteps {
        let t = t as f64 * 1f64 / (nsteps as f64);
        let p = bezier.eval_casteljau(t);
        bezier_graph.push((p.axis(0), p.axis(1)));
        let p = bezier.eval(t);
        bezier_graph_reg.push((p.axis(0), p.axis(1)));
    }

    let root =
        BitMapBackend::new("cubic_bezier_bounding_box.png", (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    // setup the chart
    let mut chart = ChartBuilder::on(&root)
        .caption("Cubic Bezier Curve", ("sans-serif", 21).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            (xmin - 2.0)..(xmin + dmax + 2.0),
            (ymin - 2.0)..(ymin + dmax + 2.0),
        )?; // make graph a bit bigger than bounding box

    chart.configure_mesh().draw()?;

    // draw the control points of B(t)
    chart
        .draw_series(PointSeries::of_element(
            cpoints.clone(),
            5,
            &BLUE,
            &|coord, size, style| {
                EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                    + Text::new(
                        format!("{:?}", coord),
                        (0, 15),
                        ("sans-serif", 15).into_font(),
                    )
            },
        ))?
        .label("Control Points of B(t)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    // draw the actual bezier curve
    chart
        .draw_series(LineSeries::new(bezier_graph, &RED))?
        .label("B(t) castlejau")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    chart
        .draw_series(LineSeries::new(bezier_graph_reg, &RED))?
        .label("B(t)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    chart
        .draw_series(LineSeries::new([cpoints[0], cpoints[1]], &BLUE))?
        .label("handle 0")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .draw_series(LineSeries::new([cpoints[3], cpoints[2]], &BLUE))?
        .label("handle 1")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    // draw the bounding box
    chart
        .draw_series(
            AreaSeries::new(
                vec![
                    (xmin, ymin),
                    (xmin, ymax),
                    (xmax, ymax),
                    (xmax, ymin),
                    (xmin, ymin),
                ],
                0.0,
                GREEN.mix(0.05),
            )
            .border_style(GREEN),
        )?
        .label("Bounding Box")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    Ok(())
}
