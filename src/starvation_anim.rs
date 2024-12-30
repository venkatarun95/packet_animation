use plotters::prelude::*;
use std::error::Error;

fn cca_behavior(link_rate: f64) -> Vec<(f64, f64)> {
    assert!(link_rate >= 1. / 9. && link_rate <= 1.);
    let mut res = Vec::new();
    let ss_exit = 1. / link_rate;
    let mut slow_start = true;
    // Index at which slow start ended
    let mut ss_ended = 0;
    let mut prev_y = 0.1;
    for i in 0..1000 {
        let x = 10. * i as f64 / 1000.;
        res.push((x, prev_y));
        if slow_start {
            prev_y += 0.02 * prev_y;
            if prev_y > ss_exit {
                slow_start = false;
                ss_ended = i;
            }
        } else {
            let i = i - ss_ended;
            // prev_y += if (i / 100) % 2 == 0 { 1. } else { -1. } * (1. / link_rate) / (10. * 100.);
            prev_y += if (i / 100) % 2 == 0 { -1. } else { 1. } * (5.0) / (10. * 100.);
        }
    }
    res
}

pub fn starvation_anim() -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::gif("starvation-knob.gif", (1200, 800), 16)?.into_drawing_area();
    let (graph, knob) = root.split_vertically(600);
    const NUM_FRAMES: usize = 33 * 9;

    let grey = ShapeStyle {
        color: RGBAColor(128, 128, 128, 1.0),
        filled: false,
        stroke_width: 1,
    };
    let highlight = ShapeStyle {
        color: RGBAColor(68, 114, 196, 1.0),
        filled: false,
        stroke_width: 2,
    };

    for frame in 0..NUM_FRAMES {
        graph.fill(&WHITE)?;
        knob.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&graph).build_cartesian_2d(0.0..10.0, 0.0..10.0)?;
        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .y_desc("Time")
            .x_desc("Packet delay")
            .draw()?;

        // let link_rate = 0.1 + 0.9 * frame as f64 / NUM_FRAMES as f64;
        let link_rate = 1. / 9. + ((6.28 * frame as f64 / NUM_FRAMES as f64).sin() + 1.) * 0.42;

        chart.draw_series(LineSeries::new(cca_behavior(link_rate), highlight))?;

        let mut knob_chart = ChartBuilder::on(&knob).build_cartesian_2d(-0.1..1.1, -1f32..3f32)?;
        knob_chart.draw_series(vec![Circle::new(
            (link_rate, 0.0),
            15.,
            Into::<ShapeStyle>::into(&BLACK).filled(),
        )])?;
        knob_chart.draw_series(vec![Rectangle::new(
            [(0.0, -0.05), (1.0, 0.05)],
            Into::<ShapeStyle>::into(&BLACK).filled(),
        )])?;

        root.present()?;
    }

    // Plot a graph with multiple lines
    let root = BitMapBackend::new("starvation-multiple.png", (1200, 800)).into_drawing_area();
    let (graph, knob) = root.split_vertically(600);
    graph.fill(&WHITE)?;
    knob.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&graph).build_cartesian_2d(0.0..10.0, 0.0..10.0)?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .y_desc("Time")
        .x_desc("Packet delay")
        .draw()?;

    for (link_rate, style) in [
        (0.12, grey),
        (0.2, grey),
        (0.3, grey),
        (0.4, grey),
        (0.5, grey),
        (0.6, highlight),
        (0.7, grey),
        (0.8, grey),
        (0.9, grey),
        (0.95, highlight),
    ] {
        chart.draw_series(LineSeries::new(cca_behavior(link_rate), style))?;
    }

    let mut knob_chart = ChartBuilder::on(&knob).build_cartesian_2d(-0.1..1.1, -1f32..3f32)?;
    knob_chart.draw_series(vec![Circle::new(
        (0.95, 0.0),
        15.,
        Into::<ShapeStyle>::into(&BLACK).filled(),
    )])?;
    let mut knob_chart = ChartBuilder::on(&knob).build_cartesian_2d(-0.1..1.1, -1f32..3f32)?;
    knob_chart.draw_series(vec![Circle::new(
        (0.5, 0.0),
        15.,
        Into::<ShapeStyle>::into(&BLACK).filled(),
    )])?;
    knob_chart.draw_series(vec![Rectangle::new(
        [(0.0, -0.05), (1.0, 0.05)],
        Into::<ShapeStyle>::into(&BLACK).filled(),
    )])?;

    // Plot two graphs with a region around the lines
    let root = BitMapBackend::new("starvation-area.png", (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root).build_cartesian_2d(0.0..10.0, 0.0..10.0)?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .y_desc("Time")
        .x_desc("Packet delay")
        .draw()?;

    let mut poly = cca_behavior(0.5);
    let mut translated_line = poly.iter().map(|(x, y)| (*x, y - 1.0)).collect::<Vec<_>>();
    translated_line.reverse();
    poly.append(&mut translated_line);

    chart.draw_series(vec![Polygon::new(
        poly,
        ShapeStyle {
            color: RGBAColor(64, 116, 155, 1.0),
            filled: true,
            stroke_width: 0,
        },
    )])?;

    Ok(())
}
