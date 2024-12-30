use plotters::prelude::*;

pub const PKT_HEIGHT: f64 = 1.5;
pub const DATA_PKT_WIDTH: f64 = 0.25;
pub const ACK_PKT_WIDTH: f64 = 0.1;

#[derive(Clone, Copy, Debug)]
pub struct Coord(pub f64, pub f64);

impl Coord {
    pub fn add(&self, other: Coord) -> Coord {
        Coord(self.0 + other.0, self.1 + other.1)
    }

    pub fn sub(&self, other: Coord) -> Coord {
        Coord(self.0 - other.0, self.1 - other.1)
    }

    pub fn div(&self, fact: f64) -> Coord {
        Coord(self.0 / fact, self.1 / fact)
    }
}

impl From<Coord> for (f64, f64) {
    fn from(value: Coord) -> (f64, f64) {
        (value.0, value.1)
    }
}

#[derive(Clone, Copy)]
pub struct Packet {
    /// Width of the packet representing the number of bytes it has
    pub size: f64,
    /// Location of the left-center of the rectangle representing the packet
    pub coord: Coord,
    /// Address which we will use to route packets
    pub addr: u16,
    /// How to draw the packet? Color, fill, border etc.
    pub style: ShapeStyle,
}

impl Packet {
    pub fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>> {
        vec![
            Rectangle::new(
                [
                    (self.coord.0, self.coord.1 - PKT_HEIGHT / 2.),
                    (self.coord.0 + self.size, self.coord.1 + PKT_HEIGHT / 2.),
                ],
                self.style,
            )
            .into_dyn(),
            Rectangle::new(
                [
                    (self.coord.0, self.coord.1 - PKT_HEIGHT / 2.),
                    (self.coord.0 + self.size, self.coord.1 + PKT_HEIGHT / 2.),
                ],
                ShapeStyle::from(BLACK).stroke_width(2),
            )
            .into_dyn(),
        ]
    }
}

pub trait Element {
    fn get_enqueue_coord(&self) -> Coord;
    fn enqueue(&mut self, pkt: &Packet);
    fn get_pkts(&self) -> Vec<Packet>;
    fn tick(&mut self);
    // fn draw<'a, 'b>(&'a self, chart: &'b mut Chart) -> Result<(), Box<dyn std::error::Error>>;
    fn draw<DB: DrawingBackend>(&self) -> Vec<DynElement<DB, (f64, f64)>>;
}
