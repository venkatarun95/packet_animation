use crate::base::{Coord, Element, Packet};
use plotters::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Transports packets from the coordinate at which they were enqueued to the
/// coordinate returned by `next.get_enqueue_coord`
pub struct Transport<N: Element> {
    /// Time taken to traverse the area in ticks
    delay: u64,
    /// All the packets in flight. Stores (pkt, number of ticks since they were
    /// enqueued, coordinate from which they started, coordinate to which they are headed)
    pkts: Vec<(Packet, u64, Coord, Coord)>,
    next: Rc<RefCell<N>>,
}

impl<N: Element> Transport<N> {
    pub fn new(delay: u64, next: Rc<RefCell<N>>) -> Self {
        Self {
            delay,
            next,
            pkts: Vec::new(),
        }
    }
}

impl<N: Element> Element for Transport<N> {
    fn get_enqueue_coord(&self) -> Coord {
        // Transport is not supposed to supply this info
        unreachable!()
    }

    fn enqueue(&mut self, pkt: &Packet) {
        self.pkts.push((
            *pkt,
            0,
            pkt.coord,
            self.next.borrow_mut().get_enqueue_coord(),
        ));
    }

    fn get_pkts(&self) -> Vec<Packet> {
        self.pkts.iter().map(|x| x.0).collect()
    }

    fn tick(&mut self) {
        let mut to_remove = Vec::new();
        for i in 0..self.pkts.len() {
            let (_, _, start, end) = self.pkts[i];
            let speed = end.sub(start).div(self.delay as f64);

            self.pkts[i].0.coord = self.pkts[i].0.coord.add(speed);
            self.pkts[i].1 += 1;
            if self.pkts[i].1 >= self.delay {
                self.next.borrow_mut().enqueue(&self.pkts[i].0);
                to_remove.push(i);
            }
        }
        for i in to_remove {
            // Note, `swap_remove` does not preserve order of elements
            self.pkts.swap_remove(i);
        }
    }

    fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>> {
        let mut res = Vec::new();
        for pkt in &self.pkts {
            res.extend(pkt.0.draw());
        }
        res
    }
}
