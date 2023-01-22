//! Animate just a single element
use crate::base::{Coord, Element, Packet, DATA_PKT_WIDTH};
use crate::bottleneck::Bottleneck;
use crate::simple_elems::Sink;
use crate::transport::Transport;
use plotters::prelude::*;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

pub struct ElemAnimConfig {
    pub filename: String,
    /// Buffer size for both directions
    pub bufsize: u64,
    /// Sequence of intersend times (not mahimahi-like)
    pub bottleneck_intersend: Vec<u64>,
    /// Sequence of intersend times
    pub sender_intersend: Vec<u64>,
    /// Number of ticks to animate
    pub num_ticks: u64,
}

pub fn elem_anim(config: &ElemAnimConfig) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::gif(&config.filename, (300, 100), 33)?.into_drawing_area();

    let sink = Rc::new(RefCell::new(Sink {
        coord: Coord(5., 0.),
    }));
    let departure = Rc::new(RefCell::new(Transport::new(16, sink.clone())));
    let elem = Rc::new(RefCell::new(Bottleneck::new(
        Coord(0., 0.),
        config.bufsize,
        config.bottleneck_intersend.clone(),
        vec![departure.clone()],
        true,
    )));
    let arrival = Rc::new(RefCell::new(Transport::new(16, elem.clone())));

    let mut intersend_index = 0;
    let mut time_since_send = 0;
    for _tick in 0..config.num_ticks {
        root.fill(&WHITE)?;
        let chart = ChartBuilder::on(&root).build_cartesian_2d(-5.0..5.0, -5.0..5.0)?;

        // Produce packets
        {
            let mut arrival = arrival.borrow_mut();
            if time_since_send >= config.sender_intersend[intersend_index] {
                arrival.enqueue(&Packet {
                    size: DATA_PKT_WIDTH,
                    coord: Coord(-5., 0.),
                    addr: 0,
                    style: ShapeStyle::from(RED).filled(),
                });
                intersend_index = (intersend_index + 1) % config.sender_intersend.len();
                time_since_send = 0;
            }
            time_since_send += 1;
            arrival.tick();
            for e in arrival.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        {
            let mut elem = elem.borrow_mut();
            elem.tick();
            for e in elem.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        {
            let mut departure = departure.borrow_mut();
            departure.tick();
            for e in departure.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        root.present()?;
    }
    Ok(())
}
