use plotters::prelude::*;
use std::error::Error;

fn cca_behavior(link_rate: f64) -> Vec<(f64, f64)> {
    assert!(link_rate >= 1. / 9. && link_rate <= 1.);
    let mut res = Vec::new();
    let ss_exit = 1. / link_rate;
    let mut slow_start = true;
    let mut prev_y = 0.1;
    for i in 0..1000 {
        let x = 10. * i as f64 / 1000.;
        res.push((x, prev_y));
        if slow_start {
            prev_y += 0.02 * prev_y;
            if prev_y > ss_exit {
                slow_start = false;
            }
        } else {
            prev_y += if (i / 100) % 2 == 0 { 1. } else { -1. } * (1. / link_rate) / (10. * 100.);
        }
    }
    res
}

pub fn starvation_anim() -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::gif("starvation-knob.gif", (1200, 800), 33)?.into_drawing_area();
    let (graph, knob) = root.split_vertically(600);
    const NUM_FRAMES: usize = 33 * 3;

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

        chart.draw_series(LineSeries::new(
            cca_behavior(link_rate),
            Into::<ShapeStyle>::into(&BLUE).stroke_width(3),
        ))?;

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
    Ok(())
}
