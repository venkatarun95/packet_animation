//! Animate two flows fairly sharing a link
use crate::base::{Coord, Element, Packet, DATA_PKT_WIDTH, PKT_HEIGHT};
use crate::bottleneck::Bottleneck;
use crate::simple_elems::{Acker, Sink};
use crate::transport::Transport;
use plotters::prelude::*;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

pub struct FairAnimConfig {
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

pub fn fair_anim(config: &FairAnimConfig) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::gif(&config.filename, (1600, 400), 33)?.into_drawing_area();

    // Vertical separation between flows
    let vsep = PKT_HEIGHT * 13.;

    // Shared bottlenecks. We will populate `next` later after we have
    // constructed them
    let ret_bottleneck = Rc::new(RefCell::new(Bottleneck::new(
        Coord(DATA_PKT_WIDTH * config.bufsize as f64, -PKT_HEIGHT * 2.),
        config.bufsize,
        config.bottleneck_intersend,
        vec![],
        false,
    )));
    let bottleneck = Rc::new(RefCell::new(Bottleneck::new(
        Coord(0., 0.),
        config.bufsize,
        config.bottleneck_intersend,
        vec![],
        true,
    )));

    let sink_a = Rc::new(RefCell::new(Sink {
        coord: Coord(-9., -PKT_HEIGHT * 2. + vsep),
    }));
    let returnpath2_a = Rc::new(RefCell::new(Transport::new(16, sink_a.clone())));
    let returnpath1_a = Rc::new(RefCell::new(Transport::new(16, ret_bottleneck.clone())));
    let acker_a = Rc::new(RefCell::new(Acker {
        rcv_coord: Coord(9., 0. + vsep),
        snd_coord: Coord(9., -PKT_HEIGHT * 2. + vsep),
        next: returnpath1_a.clone(),
    }));
    let departure_a = Rc::new(RefCell::new(Transport::new(16, acker_a.clone())));
    let arrival_a = Rc::new(RefCell::new(Transport::new(32, bottleneck.clone())));

    let sink_b = Rc::new(RefCell::new(Sink {
        coord: Coord(-9., -PKT_HEIGHT * 2. - vsep),
    }));
    let returnpath2_b = Rc::new(RefCell::new(Transport::new(16, sink_b.clone())));
    let returnpath1_b = Rc::new(RefCell::new(Transport::new(16, ret_bottleneck.clone())));
    let acker_b = Rc::new(RefCell::new(Acker {
        rcv_coord: Coord(9., 0. - vsep),
        snd_coord: Coord(9., -PKT_HEIGHT * 2. - vsep),
        next: returnpath1_b.clone(),
    }));
    let departure_b = Rc::new(RefCell::new(Transport::new(16, acker_b.clone())));
    let arrival_b = Rc::new(RefCell::new(Transport::new(32, bottleneck.clone())));

    ret_bottleneck
        .borrow_mut()
        .set_next(vec![returnpath2_a.clone(), returnpath2_b.clone()]);
    bottleneck
        .borrow_mut()
        .set_next(vec![departure_a.clone(), departure_b.clone()]);

    let mut num_packets = 0;
    for tick in 0..config.num_ticks {
        root.fill(&WHITE)?;
        let chart = ChartBuilder::on(&root).build_cartesian_2d(-10.0..10.0, -20.0..20.0)?;

        // Produce packets
        {
            let mut arrival_a = arrival_a.borrow_mut();
            if tick % config.sender_intersend == 0
                || (tick % (config.sender_intersend / 2) == 0
                    && num_packets < config.num_extra_packets)
            {
                arrival_a.enqueue(&Packet {
                    size: DATA_PKT_WIDTH,
                    coord: Coord(-10., vsep),
                    addr: 0,
                    style: ShapeStyle::from(RED).filled(),
                });
                num_packets += 1;
            }
            arrival_a.tick();
            for e in arrival_a.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        {
            let mut arrival_b = arrival_b.borrow_mut();
            if (tick + config.sender_intersend / 2) % config.sender_intersend == 0
                || ((tick + config.sender_intersend / 2) % (config.sender_intersend / 2) == 0
                    && num_packets < config.num_extra_packets)
            {
                arrival_b.enqueue(&Packet {
                    size: DATA_PKT_WIDTH,
                    coord: Coord(-10., -vsep),
                    addr: 1,
                    style: ShapeStyle::from(GREEN).filled(),
                });
                num_packets += 1;
            }
            arrival_b.tick();
            for e in arrival_b.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        // Shared bottleneck
        {
            let mut bottleneck = bottleneck.borrow_mut();
            bottleneck.tick();
            for e in bottleneck.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        {
            let mut ret_bottleneck = ret_bottleneck.borrow_mut();
            ret_bottleneck.tick();
            for e in ret_bottleneck.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        // Flow A
        {
            let mut acker_a = acker_a.borrow_mut();
            acker_a.tick();
            for e in acker_a.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        {
            let mut departure_a = departure_a.borrow_mut();
            departure_a.tick();
            for e in departure_a.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        {
            let mut returnpath1_a = returnpath1_a.borrow_mut();
            returnpath1_a.tick();
            for e in returnpath1_a.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        {
            let mut returnpath2_a = returnpath2_a.borrow_mut();
            returnpath2_a.tick();
            for e in returnpath2_a.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        // Flow B
        {
            let mut departure_b = departure_b.borrow_mut();
            departure_b.tick();
            for e in departure_b.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        {
            let mut acker_b = acker_b.borrow_mut();
            acker_b.tick();
            for e in acker_b.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        {
            let mut returnpath1_b = returnpath1_b.borrow_mut();
            returnpath1_b.tick();
            for e in returnpath1_b.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        {
            let mut returnpath2_b = returnpath2_b.borrow_mut();
            returnpath2_b.tick();
            for e in returnpath2_b.draw() {
                chart.plotting_area().draw(&e)?;
            }
        }

        root.present()?;
    }
    Ok(())
}
