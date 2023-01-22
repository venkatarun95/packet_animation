//! Animation of the entire path
use crate::base::{Coord, Element, Packet, DATA_PKT_WIDTH, PKT_HEIGHT};
use crate::bottleneck::Bottleneck;
use crate::simple_elems::{Acker, Sink};
use crate::transport::Transport;
use plotters::prelude::*;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

pub struct TwoBottlenecksAnimConfig {
    /// Where to output the .gif file
    pub filename: String,
    /// Buffer size for both directions
    pub bufsize1: u64,
    pub bufsize2: u64,
    /// Bottleneck link rate
    pub bottleneck1_intersend: Vec<u64>,
    /// Bottleneck link rate
    pub bottleneck2_intersend: Vec<u64>,
    /// Sending rate
    pub sender_intersend: u64,
    /// Number of extra packets to send beyond sender_intersend
    pub num_extra_packets: u64,
    /// Number of ticks to animate
    pub num_ticks: u64,
    /// Should we draw the buffer for the two bottlenecks?
    pub draw_buffer: (bool, bool),
}

pub fn two_bottlenecks_anim(config: &TwoBottlenecksAnimConfig) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::gif(&config.filename, (1600, 100), 33)?.into_drawing_area();

    let sink = Rc::new(RefCell::new(Sink {
        coord: Coord(-9., -PKT_HEIGHT * 2.),
    }));
    let returnpath2 = Rc::new(RefCell::new(Transport::new(16, sink.clone())));
    let ret_bottleneck = Rc::new(RefCell::new(Bottleneck::new(
        Coord(
            DATA_PKT_WIDTH * config.bufsize1 as f64 - 3.33,
            -PKT_HEIGHT * 2.,
        ),
        config.bufsize1,
        config.bottleneck1_intersend.clone(),
        vec![returnpath2.clone()],
        false,
    )));
    let returnpath1 = Rc::new(RefCell::new(Transport::new(16, ret_bottleneck.clone())));
    let acker = Rc::new(RefCell::new(Acker {
        rcv_coord: Coord(9., 0.),
        snd_coord: Coord(9., -PKT_HEIGHT * 2.),
        next: returnpath1.clone(),
    }));
    let departure = Rc::new(RefCell::new(Transport::new(16, acker.clone())));
    let bottleneck2 = Rc::new(RefCell::new(Bottleneck::new(
        Coord(3.33, 0.),
        config.bufsize2,
        config.bottleneck2_intersend.clone(),
        vec![departure.clone()],
        true,
    )));
    let between_2_bottlenecks = Rc::new(RefCell::new(Transport::new(16, bottleneck2.clone())));
    let bottleneck = Rc::new(RefCell::new(Bottleneck::new(
        Coord(-3.33, 0.),
        config.bufsize1,
        config.bottleneck1_intersend.clone(),
        vec![between_2_bottlenecks.clone()],
        true,
    )));
    let arrival = Rc::new(RefCell::new(Transport::new(32, bottleneck.clone())));

    {
        bottleneck.borrow_mut().draw_buffer(config.draw_buffer.0);
        bottleneck2.borrow_mut().draw_buffer(config.draw_buffer.1);
    }

    let mut num_packets = 0;
    for tick in 0..config.num_ticks {
        root.fill(&WHITE)?;
        let chart = ChartBuilder::on(&root).build_cartesian_2d(-10.0..10.0, -5.0..5.0)?;

        let mut arrival = arrival.borrow_mut();

        if tick % config.sender_intersend == 0
            || (tick % (config.sender_intersend / 2) == 0 && num_packets < config.num_extra_packets)
        {
            arrival.enqueue(&Packet {
                size: DATA_PKT_WIDTH,
                coord: Coord(-10., 0.),
                addr: 0,
                style: ShapeStyle::from(RED).filled(),
            });
            num_packets += 1;
        }

        arrival.tick();
        for e in arrival.draw() {
            chart.plotting_area().draw(&e)?;
        }

        let mut bottleneck = bottleneck.borrow_mut();
        bottleneck.tick();
        for e in bottleneck.draw() {
            chart.plotting_area().draw(&e)?;
        }

        let mut between_2_bottlenecks = between_2_bottlenecks.borrow_mut();
        between_2_bottlenecks.tick();
        for e in between_2_bottlenecks.draw() {
            chart.plotting_area().draw(&e)?;
        }

        let mut bottleneck2 = bottleneck2.borrow_mut();
        bottleneck2.tick();
        for e in bottleneck2.draw() {
            chart.plotting_area().draw(&e)?;
        }

        let mut departure = departure.borrow_mut();
        departure.tick();
        for e in departure.draw() {
            chart.plotting_area().draw(&e)?;
        }

        let mut acker = acker.borrow_mut();
        acker.tick();
        for e in acker.draw() {
            chart.plotting_area().draw(&e)?;
        }

        let mut returnpath1 = returnpath1.borrow_mut();
        returnpath1.tick();
        for e in returnpath1.draw() {
            chart.plotting_area().draw(&e)?;
        }

        let mut ret_bottleneck = ret_bottleneck.borrow_mut();
        ret_bottleneck.tick();
        for e in ret_bottleneck.draw() {
            chart.plotting_area().draw(&e)?;
        }

        let mut returnpath2 = returnpath2.borrow_mut();
        returnpath2.tick();
        for e in returnpath2.draw() {
            chart.plotting_area().draw(&e)?;
        }

        root.present()?;
    }
    Ok(())
}
