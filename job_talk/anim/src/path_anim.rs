//! Animation of the entire path
use crate::base::{Coord, Element, Packet, DATA_PKT_WIDTH, PKT_HEIGHT};
use crate::bottleneck::Bottleneck;
use crate::simple_elems::{Acker, Sink};
use crate::transport::Transport;
use plotters::prelude::*;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

pub struct PathAnimConfig {
    /// Where to output the .gif file
    pub filename: String,
    /// Buffer size for both directions
    pub bufsize: u64,
    /// Bottleneck link rate
    pub bottleneck_intersend: u64,
    /// Sending rate
    pub sender_intersend: u64,
    /// Number of extra packets to send beyond sender_intersend
    pub num_extra_packets: u64,
    /// Number of ticks to animate
    pub num_ticks: u64,
}

pub fn path_anim(config: &PathAnimConfig) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::gif(&config.filename, (1600, 100), 33)?.into_drawing_area();

    let sink = Rc::new(RefCell::new(Sink {
        coord: Coord(-9., -PKT_HEIGHT * 2.),
    }));
    let returnpath2 = Rc::new(RefCell::new(Transport::new(16, sink.clone())));
    let ret_bottleneck = Rc::new(RefCell::new(Bottleneck::new(
        Coord(DATA_PKT_WIDTH * config.bufsize as f64, -PKT_HEIGHT * 2.),
        config.bufsize,
        config.bottleneck_intersend,
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
    let bottleneck = Rc::new(RefCell::new(Bottleneck::new(
        Coord(0., 0.),
        config.bufsize,
        config.bottleneck_intersend,
        vec![departure.clone()],
        true,
    )));
    let arrival = Rc::new(RefCell::new(Transport::new(32, bottleneck.clone())));

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
