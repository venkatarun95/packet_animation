mod base;
mod bottleneck;
mod simple_elems;
mod transport;

use crate::base::{Coord, Element, Packet, DATA_PKT_WIDTH, PKT_HEIGHT};
use crate::bottleneck::Bottleneck;
use crate::simple_elems::{Acker, Sink};
use crate::transport::Transport;
use plotters::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::gif("ideal.gif", (800, 300), 33)?.into_drawing_area();

    let sink = Rc::new(RefCell::new(Sink {
        coord: Coord(-9., -PKT_HEIGHT * 2.),
    }));
    let returnpath2 = Rc::new(RefCell::new(Transport::new(16, sink.clone())));
    let ret_bottleneck = Rc::new(RefCell::new(Bottleneck::new(
        Coord(DATA_PKT_WIDTH * 10., -PKT_HEIGHT * 2.),
        10,
        8,
        returnpath2.clone(),
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
        10,
        16,
        departure.clone(),
        true,
    )));
    let arrival = Rc::new(RefCell::new(Transport::new(32, bottleneck.clone())));

    for tick in 0..640 {
        root.fill(&WHITE)?;
        let chart = ChartBuilder::on(&root).build_cartesian_2d(-10.0..10.0, -5.0..5.0)?;

        let mut arrival = arrival.borrow_mut();

        if tick % 8 == 0 {
            arrival.enqueue(&Packet {
                size: DATA_PKT_WIDTH,
                coord: Coord(-10., 0.),
                addr: 0,
                style: RED.into(),
            });
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
