use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_workers::<16>()
        .set_ring_buffer_size::<128>()
        .build()
        .unwrap();

    for i in 0..128 {
        cahotic.spawn_task(DefaultTask(|| {
            // println!("done");
            DefaultOutput(10)
        }));
    }

    cahotic.join();
}
