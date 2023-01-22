mod base;
mod bottleneck;
mod elem_anim;
mod fair_anim;
mod path_anim;
mod simple_elems;
mod starvation_anim;
mod transport;
mod two_bottlenecks_anim;

use crate::elem_anim::{elem_anim, ElemAnimConfig};
use crate::fair_anim::{fair_anim, FairAnimConfig};
use crate::path_anim::{path_anim, PathAnimConfig};
use crate::starvation_anim::starvation_anim;
use crate::two_bottlenecks_anim::{two_bottlenecks_anim, TwoBottlenecksAnimConfig};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    starvation_anim()?;

    let two_bottlenecks_config = TwoBottlenecksAnimConfig {
        filename: String::from("two-bottlenecks-ideal.gif"),
        bufsize1: 8,
        bottleneck1_intersend: vec![10],
        bufsize2: 4,
        bottleneck2_intersend: vec![10],
        sender_intersend: 10,
        num_extra_packets: 7,
        num_ticks: 640,
    };
    two_bottlenecks_anim(&two_bottlenecks_config)?;

    let adversary_config = TwoBottlenecksAnimConfig {
        filename: String::from("two-bottlenecks-adversary.gif"),
        bufsize1: 8,
        bottleneck1_intersend: vec![10],
        bufsize2: 4,
        //bottleneck2_intersend: vec![5, 0, 0, 20, 10, 0, 30, 15],
        bottleneck2_intersend: vec![0, 0, 30],
        sender_intersend: 10,
        num_extra_packets: 7,
        num_ticks: 640,
    };
    two_bottlenecks_anim(&adversary_config)?;

    let elem_config = ElemAnimConfig {
        filename: String::from("elem-ideal.gif"),
        bufsize: 8,
        bottleneck_intersend: vec![10],
        sender_intersend: vec![10],
        num_ticks: 640,
    };
    elem_anim(&elem_config)?;

    let elem_config = ElemAnimConfig {
        filename: String::from("elem-agg.gif"),
        bufsize: 8,
        bottleneck_intersend: vec![70, 1, 1, 1, 1, 1, 1],
        sender_intersend: vec![11],
        num_ticks: 640,
    };
    elem_anim(&elem_config)?;

    let elem_config = ElemAnimConfig {
        filename: String::from("elem-random.gif"),
        bufsize: 10,
        //bottleneck_intersend: vec![10, 5, 2, 15, 20, 2, 12, 12, 12, 5],
        bottleneck_intersend: vec![5, 1, 1, 1, 1, 12, 20, 1, 10, 6, 6],
        sender_intersend: vec![6],
        num_ticks: 640,
    };
    elem_anim(&elem_config)?;

    let elem_config = ElemAnimConfig {
        filename: String::from("elem-tbf.gif"),
        bufsize: 10,
        //bottleneck_intersend: vec![10, 5, 2, 15, 20, 2, 12, 12, 12, 5],
        bottleneck_intersend: vec![1, 1, 1, 10, 10, 10, 10, 10, 10],
        sender_intersend: vec![1, 1, 1, 1, 1, 1, 1, 1, 90],
        num_ticks: 640,
    };
    elem_anim(&elem_config)?;

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
