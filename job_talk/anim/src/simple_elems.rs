use crate::base::{Coord, Element, Packet, ACK_PKT_WIDTH};
use plotters::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Does nothing. Simply sinks packets
pub struct Sink {
    pub coord: Coord,
}

impl Element for Sink {
    fn get_enqueue_coord(&self) -> Coord {
        self.coord
    }
    fn enqueue(&mut self, _: &Packet) {}
    fn get_pkts(&self) -> Vec<Packet> {
        Vec::new()
    }
    fn tick(&mut self) {}
    fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>> {
        Vec::new()
    }
}

/// Simply acks packets back
pub struct Acker<N: Element> {
    pub rcv_coord: Coord,
    pub snd_coord: Coord,
    pub next: Rc<RefCell<N>>,
}

impl<N: Element> Element for Acker<N> {
    fn get_enqueue_coord(&self) -> Coord {
        self.rcv_coord
    }
    fn enqueue(&mut self, pkt: &Packet) {
        let mut pkt = pkt.clone();
        pkt.size = ACK_PKT_WIDTH;
        pkt.coord = self.snd_coord;
        self.next.borrow_mut().enqueue(&pkt);
    }
    fn get_pkts(&self) -> Vec<Packet> {
        Vec::new()
    }
    fn tick(&mut self) {}
    fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>> {
        Vec::new()
    }
}
