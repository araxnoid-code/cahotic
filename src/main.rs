use std::{
    sync::atomic::{AtomicIsize, AtomicU32},
    u64,
};

fn main() {
    // let bitmap = u64::MAX;
    let data = AtomicU32::new(0b10000000000000000000000000010001);
    let target = 4_u32;
    let mark = !(1_u32 << target);
    println!("{:032b}", data.load(std::sync::atomic::Ordering::Acquire));
    println!("{:032b}", mark);

    // data.fetch_and(mark, std::sync::atomic::Ordering::Release);
    // println!("{:032b}", data.load(std::sync::atomic::Ordering::Acquire));

    // let index = AtomicIsize::new(1);
    // println!(
    //     "{}",
    //     index.fetch_add(1, std::sync::atomic::Ordering::Release)
    // );
    // println!(
    //     "{}",
    //     index.fetch_add(1, std::sync::atomic::Ordering::Release)
    // )
}
