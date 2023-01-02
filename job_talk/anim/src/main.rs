use plotters::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

// use print_type_of on Chart to get this type
type Chart<'a> = plotters::chart::ChartContext<
    'a,
    BitMapBackend<'a>,
    Cartesian2d<plotters::coord::types::RangedCoordf64, plotters::coord::types::RangedCoordf64>,
>;

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
        println!("{self:?}");
        vec![Rectangle::new(
            [
                (self.coord.0, self.coord.1 - 0.5),
                (self.coord.0 + self.size, self.coord.1 + 0.5),
            ],
            &RED,
        )
        .into_dyn()]
    }
    // fn draw(&self, chart: &mut Chart) -> Result<(), Box<dyn std::error::Error>> {
    //     let rect: Vec<(f64, f64)> = vec![
    //         self.coord.add(Coord(0.0, 0.5)).into(),
    //         self.coord.add(Coord(0.0, -0.5)).into(),
    //         self.coord.add(Coord(self.size, -0.5)).into(),
    //         self.coord.add(Coord(self.size, 0.5)).into(),
    //     ];
    //     //chart.draw_series(std::iter::once(PathElement::new(rect, &RED)))?;
    //     chart.draw_series(std::iter::once(Polygon::new(rect, &RED.mix(0.2))))?;
    //     Ok(())
    // }
}

trait Element {
    fn get_enqueue_coord(&self) -> Coord;
    fn enqueue(&mut self, pkt: &Packet);
    fn get_pkts(&self) -> Vec<Packet>;
    fn tick(&mut self);
    // fn draw<'a, 'b>(&'a self, chart: &'b mut Chart) -> Result<(), Box<dyn std::error::Error>>;
    fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>>;
}

struct Transport<N: Element> {
    start: Coord,
    end: Coord,
    /// Time taken to traverse the area in ticks
    delay: u64,
    /// All the packets in flight and the number of ticks since they were enqueued
    pkts: Vec<(Packet, u64)>,
    next: Rc<RefCell<N>>,
}

impl<N: Element> Transport<N> {
    fn new(start: Coord, end: Coord, delay: u64, next: Rc<RefCell<N>>) -> Self {
        Self {
            start,
            end,
            delay,
            next,
            pkts: Vec::new(),
        }
    }
}

impl<N: Element> Element for Transport<N> {
    fn get_enqueue_coord(&self) -> Coord {
        self.start
    }

    fn enqueue(&mut self, pkt: &Packet) {
        let mut pkt = pkt.clone();
        pkt.coord = self.start;
        self.pkts.push((pkt, 0));
    }

    fn get_pkts(&self) -> Vec<Packet> {
        self.pkts.iter().map(|x| x.0).collect()
    }

    fn tick(&mut self) {
        let speed = self.end.sub(self.start).div(self.delay as f64);

        let mut to_remove = Vec::new();
        for i in 0..self.pkts.len() {
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

    // fn draw(&self, chart: &mut Chart) -> Result<(), Box<dyn std::error::Error>> {
    //     for pkt in &self.pkts {
    //         pkt.0.draw(chart)?;
    //     }
    //     Ok(())
    // }

    fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>> {
        let mut res = Vec::new();
        for pkt in &self.pkts {
            res.extend(pkt.0.draw());
        }
        res
    }
}

/// Does nothing. Simply sinks packets
struct Sink {}

impl Element for Sink {
    fn get_enqueue_coord(&self) -> Coord {
        Coord(0., 0.)
    }
    fn enqueue(&mut self, _: &Packet) {}
    fn get_pkts(&self) -> Vec<Packet> {
        Vec::new()
    }
    fn tick(&mut self) {}
    fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>> {
        Vec::new()
        // EmptyElement::at((0., 0.)).into_dyn()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::gif("ideal.gif", (800, 600), 33)?.into_drawing_area();

    let sink = Rc::new(RefCell::new(Sink {}));
    let delay = Rc::new(RefCell::new(Transport::new(
        Coord(-9., 0.),
        Coord(9., 0.),
        64,
        sink,
    )));

    for tick in 0..64 {
        root.fill(&WHITE)?;
        let chart = ChartBuilder::on(&root).build_cartesian_2d(-10.0..10.0, -10.0..10.0)?;
        // print_type_of(&chart);

        let mut delay = delay.borrow_mut();
        if tick % 16 == 0 {
            delay.enqueue(&Packet {
                size: 1.,
                coord: Coord(0., 0.),
            });
        }
        delay.tick();
        for elem in delay.draw() {
            chart.plotting_area().draw(&elem)?;
        }

        root.present()?;
    }
    Ok(())
}
