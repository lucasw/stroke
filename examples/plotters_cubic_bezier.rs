extern crate plotters;
use plotters::prelude::*;

extern crate stroke;
use stroke::f32::CubicBezier;
use stroke::f32::Point;
use stroke::f32::PointN;

type Float = f32;

struct Curve {
    pub bezier: CubicBezier<PointN<2>, 2>,
    pub xmin: Float,
    pub ymin: Float,
    pub xmax: Float,
    pub ymax: Float,
}

impl Curve {
    fn default(cpoints: &[(Float, Float)]) -> Self {
        let bezier = CubicBezier::new(
            PointN::new([cpoints[0].0, cpoints[0].1]),
            PointN::new([cpoints[1].0, cpoints[1].1]),
            PointN::new([cpoints[2].0, cpoints[2].1]),
            PointN::new([cpoints[3].0, cpoints[3].1]),
        );

        let bounds: [_; 2] = bezier.bounding_box();
        let xmin = bounds[0].0;
        let xmax = bounds[0].1;
        let ymin = bounds[1].0;
        let ymax = bounds[1].1;

        Self {
            bezier,
            xmin,
            ymin,
            xmax,
            ymax,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // control points for the cubic bezier curve
    // let cpoints = [(0.0, 1.77), (1.1, -1.0), (5.3, 1.4), (3.2, -4.0)];
    // let cpoints = [(0.0, 0.0), (2.0, 4.0), (9.0, -1.0), (10.0, 2.0)];
    let cpoints = [(0.0, 0.0), (2.0, 0.0), (8.0, 0.0), (10.0, 0.0)];
    // let cpoints = [(0.0, 0.0), (4.0, 2.0), (0.0, 8.0), (0.0, 10.0)];

    let curve = Curve::default(&cpoints);
    let dx = curve.xmax - curve.xmin;
    let dy = curve.ymax - curve.ymin;
    let dmax = dx.max(dy);

    // render the paths of the curve to desired accuracy
    let nsteps: usize = 1000;
    let mut bezier_graph: Vec<(Float, Float)> = Vec::with_capacity(nsteps);
    let mut bezier_graph_reg: Vec<(Float, Float)> = Vec::with_capacity(nsteps);
    for ind in 0..nsteps {
        let t = ind as Float * 1.0 / (nsteps as Float);
        let p = curve.bezier.eval_casteljau(t);
        bezier_graph.push((p.axis(0), p.axis(1)));
        let p = curve.bezier.eval(t);
        bezier_graph_reg.push((p.axis(0), p.axis(1)));
    }

    let root =
        BitMapBackend::new("cubic_bezier_bounding_box.png", (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    // TODO(lucasw) trying to draw a point outside the area results in the axis getting clipped
    // to be on the edge of the area, which results in really wrong plots- is that a setting?
    // setup the chart
    let mut chart = ChartBuilder::on(&root)
        .caption("Cubic Bezier Curve", ("sans-serif", 21).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            (curve.xmin - 2.0)..(curve.xmin + dmax + 2.0),
            (curve.ymin - 2.0)..(curve.ymin + dmax + 2.0),
        )?; // make graph a bit bigger than bounding box

    chart.configure_mesh().draw()?;

    fn legend_pt(x: i32, y: i32) -> Vec<(i32, i32)> {
        vec![(x, y), (x + 20, y)]
    }

    // draw the bounding box
    chart
        .draw_series(
            AreaSeries::new(
                vec![
                    (curve.xmin, curve.ymin),
                    (curve.xmin, curve.ymax),
                    (curve.xmax, curve.ymax),
                    (curve.xmax, curve.ymin),
                    (curve.xmin, curve.ymin),
                ],
                0.0,
                GREEN.mix(0.03),
            )
            .border_style(GREEN),
        )?
        .label("Bounding Box")
        .legend(|(x, y)| PathElement::new(legend_pt(x, y), GREEN));

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
        .legend(|(x, y)| PathElement::new(legend_pt(x, y), BLUE));

    // draw the actual bezier curve
    chart
        .draw_series(LineSeries::new(bezier_graph, &RED))?
        .label(format!(
            "B(t) castlejau, length: {:.2}",
            curve.bezier.arclen(32)
        ))
        .legend(|(x, y)| PathElement::new(legend_pt(x, y), RED));

    chart
        .draw_series(LineSeries::new(bezier_graph_reg, &RED))?
        .label("B(t)")
        .legend(|(x, y)| PathElement::new(legend_pt(x, y), RED));

    {
        let point_off_line = PointN::new([3.1, 2.5]);
        // let point_off_line = PointN::new([2.5, 3.1]);
        let (test_point, test_t, distance) = curve.bezier.closest_to_point(point_off_line);
        /*
        let test_t = 0.32;
        let test_point = curve.bezier.eval(test_t);
        let distance = test_point.distance(&point_off_line);
        */
        println!("test point    {test_point:?}, t {test_t:.2}");

        let curvature = curve.bezier.curvature(test_t);
        let tangent = curve.bezier.tangent(test_t);
        println!("tangent       {tangent:?}");
        let normal = PointN::new([-tangent.axis(1), tangent.axis(0)]);
        println!("normal        {normal:?}");

        let tangent_point = test_point + tangent;

        let turn_center = {
            let rel_center = {
                let min_curvature = 0.5;
                if curvature.abs() > min_curvature {
                    let radius = 1.0 / curvature;
                    normal * radius
                } else {
                    normal * (1.0 / min_curvature)
                }
            };
            println!("rel center    {rel_center:?}");
            test_point + rel_center
        };
        println!("turn center   {turn_center:?}");

        println!("tangent point {tangent_point:?}");

        chart.draw_series(PointSeries::of_element(
            [(point_off_line.axis(0), point_off_line.axis(1))],
            5,
            &RED,
            &|coord, size, style| {
                EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                    + Text::new(
                        format!("point off line {point_off_line:?}, distance {distance:.1}",),
                        (0, 15),
                        ("sans-serif", 15).into_font(),
                    )
            },
        ))?;

        chart.draw_series(LineSeries::new(
            [
                (test_point.axis(0), test_point.axis(1)),
                (point_off_line.axis(0), point_off_line.axis(1)),
            ],
            &RED,
        ))?;

        chart.draw_series(PointSeries::of_element(
            [(test_point.axis(0), test_point.axis(1))],
            5,
            &BLUE,
            &|coord, size, style| {
                EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                    + Text::new(
                        format!(
                            "test point {:.2} {:.2} t = {test_t:.2}, curvature {curvature:.2}, radius {:.2}, tangent: {tangent:?}",
                            test_point.axis(0),
                            test_point.axis(1),
                            1.0 / curvature
                        ),
                        (-100, 15),
                        ("sans-serif", 15).into_font(),
                    )
            },
        ))?;

        chart.draw_series(LineSeries::new(
            [
                (turn_center.axis(0), turn_center.axis(1)),
                (test_point.axis(0), test_point.axis(1)),
                (tangent_point.axis(0), tangent_point.axis(1)),
            ],
            &BLUE,
        ))?;

        chart.draw_series(PointSeries::of_element(
            [(turn_center.axis(0), turn_center.axis(1))],
            5,
            &BLUE,
            &|coord, size, style| {
                EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                    + Text::new(
                        // TODO(lucasw) as mentioned above there is a serious issue with plotters
                        // drawing out-of-area lines and circles, they are moved to the boundaries
                        // but only on the one axis that went outside the boundary
                        format!(
                            "turn center {:.2} {:.2}, normal: {:.2} {:.2}",
                            turn_center.axis(0),
                            turn_center.axis(1),
                            normal.axis(0),
                            normal.axis(1)
                        ),
                        (-200, 15),
                        ("sans-serif", 15).into_font(),
                    )
            },
        ))?;

        for t in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9] {
            let point = curve.bezier.eval(t);
            chart.draw_series(PointSeries::of_element(
                [(point.axis(0), point.axis(1))],
                5,
                &RED,
                &|coord, size, style| EmptyElement::at(coord) + Circle::new((0, 0), size, style),
            ))?;
        }
    }

    chart
        .draw_series(LineSeries::new([cpoints[0], cpoints[1]], &BLUE))?
        .label("handle 0")
        .legend(|(x, y)| PathElement::new(legend_pt(x, y), BLUE));

    chart
        .draw_series(LineSeries::new([cpoints[3], cpoints[2]], &BLUE))?
        .label("handle 1")
        .legend(|(x, y)| PathElement::new(legend_pt(x, y), BLUE));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    Ok(())
}
