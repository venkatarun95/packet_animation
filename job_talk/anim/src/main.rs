mod base;
mod bottleneck;
mod fair_anim;
mod path_anim;
mod simple_elems;
mod transport;

use crate::fair_anim::{fair_anim, FairAnimConfig};
use crate::path_anim::{path_anim, PathAnimConfig};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let fair_config = FairAnimConfig {
        filename: String::from("fair.gif"),
        bufsize: 8,
        bottleneck_intersend: 5,
        sender_intersend: 10,
        num_extra_packets: 8,
        num_ticks: 640,
    };
    fair_anim(&fair_config)?;

    let path_config = PathAnimConfig {
        filename: String::from("ideal-slow.gif"),
        bufsize: 8,
        bottleneck_intersend: 10,
        sender_intersend: 20,
        num_extra_packets: 0,
        num_ticks: 640,
    };
    path_anim(&path_config)?;

    let path_config = PathAnimConfig {
        filename: String::from("ideal-correct.gif"),
        bufsize: 8,
        bottleneck_intersend: 10,
        sender_intersend: 10,
        num_extra_packets: 4,
        num_ticks: 640,
    };
    path_anim(&path_config)?;

    let path_config = PathAnimConfig {
        filename: String::from("ideal-fast.gif"),
        bufsize: 8,
        bottleneck_intersend: 10,
        sender_intersend: 5,
        num_extra_packets: 0,
        num_ticks: 640,
    };
    path_anim(&path_config)?;

    Ok(())
}
