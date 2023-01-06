use plotters::prelude::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const PKT_HEIGHT: f64 = 1.0;
const DATA_PKT_WIDTH: f64 = 0.5;
const ACK_PKT_WIDTH: f64 = 0.2;

#[derive(Clone, Copy, Debug)]
struct Coord(f64, f64);

impl Coord {
    fn add(&self, other: Coord) -> Coord {
        Coord(self.0 + other.0, self.1 + other.1)
    }

    fn sub(&self, other: Coord) -> Coord {
        Coord(self.0 - other.0, self.1 - other.1)
    }

    fn div(&self, fact: f64) -> Coord {
        Coord(self.0 / fact, self.1 / fact)
    }
}

impl From<Coord> for (f64, f64) {
    fn from(value: Coord) -> (f64, f64) {
        (value.0, value.1)
    }
}

#[derive(Clone, Copy, Debug)]
struct Packet {
    /// Width of the packet representing the number of bytes it has
    pub size: f64,
    /// Location of the left-center of the rectangle representing the packet
    pub coord: Coord,
}

impl Packet {
    fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>> {
        vec![Rectangle::new(
            [
                (self.coord.0, self.coord.1 - 0.5),
                (self.coord.0 + self.size, self.coord.1 + 0.5),
            ],
            &RED,
        )
        .into_dyn()]
    }
}

trait Element {
    fn get_enqueue_coord(&self) -> Coord;
    fn enqueue(&mut self, pkt: &Packet);
    fn get_pkts(&self) -> Vec<Packet>;
    fn tick(&mut self);
    // fn draw<'a, 'b>(&'a self, chart: &'b mut Chart) -> Result<(), Box<dyn std::error::Error>>;
    fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>>;
}

/// Transports packets from the coordinate at which they were enqueued to the
/// coordinate returned by `next.get_enqueue_coord`
struct Transport<N: Element> {
    /// Time taken to traverse the area in ticks
    delay: u64,
    /// All the packets in flight. Stores (pkt, number of ticks since they were
    /// enqueued, coordinate from which they started, coordinate to which they are headed)
    pkts: Vec<(Packet, u64, Coord, Coord)>,
    next: Rc<RefCell<N>>,
}

impl<N: Element> Transport<N> {
    fn new(delay: u64, next: Rc<RefCell<N>>) -> Self {
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

/// Does nothing. Simply sinks packets
struct Sink {
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
struct Acker<N: Element> {
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

struct Bottleneck<N: Element> {
    /// Coords of the left center of the buffer
    coord: Coord,
    /// Number of packets in buffer. Also determines visual size
    bufsize: u64,
    /// Number of ticks between successive packet transmissions
    intersend_time: u64,
    pkts: VecDeque<Packet>,
    /// To determine when to send next packet
    time_since_last_deque: u64,
    /// Used to drop packets
    dropper: Transport<Sink>,
    next: Rc<RefCell<N>>,
    /// +1 means left to right, -1 means right to left
    dir: f64,
}

impl<N: Element> Bottleneck<N> {
    /// dir = true means left to right and false means right to left
    fn new(
        coord: Coord,
        bufsize: u64,
        intersend_time: u64,
        next: Rc<RefCell<N>>,
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
            pkts: VecDeque::new(),
            time_since_last_deque: 0,
            dropper,
            next,
            dir: if dir { 1.0 } else { -1.0 },
        }
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
    }

    fn get_pkts(&self) -> Vec<Packet> {
        self.pkts.iter().map(|p| *p).collect()
    }

    fn tick(&mut self) {
        self.time_since_last_deque += 1;
        if self.time_since_last_deque >= self.intersend_time && self.pkts.len() > 0 {
            self.time_since_last_deque = 0;
            let popped = self.pkts.pop_front().unwrap();
            self.next.borrow_mut().enqueue(&popped);
            for mut pkt in &mut self.pkts {
                pkt.coord.0 += popped.size * self.dir;
            }
        }
        self.dropper.tick()
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
        for pkt in &self.pkts {
            res.extend(pkt.draw());
        }
        res.extend(self.dropper.draw());
        res
    }
}

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
