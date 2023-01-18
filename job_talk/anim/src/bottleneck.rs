use crate::base::{Coord, Element, Packet, DATA_PKT_WIDTH, PKT_HEIGHT};
use crate::simple_elems::Sink;
use crate::transport::Transport;
use plotters::prelude::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

pub struct Bottleneck<N: Element> {
    /// Coords of the left center of the buffer
    coord: Coord,
    /// Number of packets in buffer. Also determines visual size
    bufsize: u64,
    /// Number of ticks between successive packet transmissions. Note: this is
    /// different from mahimahi traces
    intersend_time: Vec<u64>,
    /// Index into intersend time
    intersend_time_index: usize,
    pkts: VecDeque<Packet>,
    /// To determine when to send next packet
    time_since_last_deque: u64,
    /// Used to drop packets
    dropper: Transport<Sink>,
    /// There can be multiple nexts. Packets go to the next as given by packet
    /// address
    next: Vec<Rc<RefCell<N>>>,
    /// +1 means left to right, -1 means right to left
    dir: f64,
    /// Amount all packets have moved to animate dequeuing
    amt_moved: f64,
    /// Little buffer of packets so we can return in `draw`; an ugly hack that
    /// is the result of poor choices with rust lifetimes. `enqueue` and `tick`
    /// copy pkt over to this. `draw` messes with this and makes it dirty.
    pkts_tmp_buffer: Vec<Packet>,
}

impl<N: Element> Bottleneck<N> {
    /// dir = true means left to right and false means right to left. Note,
    /// `next` can be empty when constructing and `next` can be filled
    /// afterward, before calling any other functions
    pub fn new(
        coord: Coord,
        bufsize: u64,
        intersend_time: Vec<u64>,
        next: Vec<Rc<RefCell<N>>>,
        dir: bool,
    ) -> Self {
        let dropper = Transport::new(
            16,
            Rc::new(RefCell::new(Sink {
                coord: coord.sub(Coord(DATA_PKT_WIDTH, 5.)),
            })),
        );

        Self {
            coord,
            bufsize,
            intersend_time,
            intersend_time_index: 0,
            pkts: VecDeque::new(),
            time_since_last_deque: 0,
            dropper,
            next,
            dir: if dir { 1.0 } else { -1.0 },
            amt_moved: 0.,
            pkts_tmp_buffer: Vec::new(),
        }
    }

    pub fn set_next(&mut self, next: Vec<Rc<RefCell<N>>>) {
        self.next = next;
    }
}

impl<N: Element> Element for Bottleneck<N> {
    fn get_enqueue_coord(&self) -> Coord {
        assert!(self.pkts.len() <= self.bufsize as usize);
        self.coord.sub(Coord(DATA_PKT_WIDTH, 0.))
    }

    fn enqueue(&mut self, pkt: &Packet) {
        if self.pkts.len() < self.bufsize as usize {
            let mut pkt = pkt.clone();
            let bufwidth: f64 = self.pkts.iter().map(|p| p.size).sum();
            pkt.coord = self
                .coord
                .sub(Coord(bufwidth * self.dir, 0.))
                .add(Coord(self.bufsize as f64 * DATA_PKT_WIDTH * self.dir, 0.))
                .sub(Coord(pkt.size * self.dir, 0.));
            self.pkts.push_back(pkt);
        } else {
            self.dropper.enqueue(pkt);
        }

        // Ugly trick for lifetimes
        self.pkts_tmp_buffer = self
            .pkts
            .iter()
            .map(|x| {
                let mut y = x.clone();
                y.coord.0 += self.amt_moved;
                y
            })
            .collect();
    }

    fn get_pkts(&self) -> Vec<Packet> {
        self.pkts.iter().map(|p| *p).collect()
    }

    fn tick(&mut self) {
        self.time_since_last_deque += 1;
        if self.time_since_last_deque >= self.intersend_time[self.intersend_time_index]
            && self.pkts.len() > 0
        {
            self.time_since_last_deque = 0;
            let popped = self.pkts.pop_front().unwrap();
            self.next[popped.addr as usize]
                .borrow_mut()
                .enqueue(&popped);
            for mut pkt in &mut self.pkts {
                pkt.coord.0 += popped.size * self.dir;
            }
            self.amt_moved = 0.;
            self.intersend_time_index = (self.intersend_time_index + 1) % self.intersend_time.len();
        }
        // Move the packets a little to indicate progress
        if let Some(_front) = self.pkts.front() {
            self.amt_moved = 0.;
            // self.amt_moved = front.size * self.time_since_last_deque as f64 / self.intersend_time as f64;
        } else {
            self.amt_moved = 0.;
        }
        self.dropper.tick();

        // Ugly trick for lifetimes
        self.pkts_tmp_buffer = self
            .pkts
            .iter()
            .map(|x| {
                let mut y = x.clone();
                y.coord.0 += self.amt_moved;
                y
            })
            .collect();
    }

    fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>> {
        let size = self.bufsize as f64 * DATA_PKT_WIDTH;
        let buffer = PathElement::new(
            vec![
                (self.coord.0, self.coord.1 - PKT_HEIGHT * 0.55 * self.dir),
                (
                    self.coord.0 + size * self.dir,
                    self.coord.1 - PKT_HEIGHT * 0.55 * self.dir,
                ),
                (
                    self.coord.0 + size * self.dir,
                    self.coord.1 + PKT_HEIGHT * 0.55 * self.dir,
                ),
                (self.coord.0, self.coord.1 + PKT_HEIGHT * 0.55 * self.dir),
            ],
            BLACK,
        );
        let mut res = vec![buffer.into_dyn()];
        // `enqueue` and `tick` nicely modified this for us
        for pkt in &self.pkts_tmp_buffer {
            res.extend(pkt.draw());
        }
        res.extend(self.dropper.draw());
        res
    }
}
