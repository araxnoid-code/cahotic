use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_workers::<8>()
        .set_ring_buffer_size::<64>()
        .build()
        .unwrap();

    for i in 0..65 {
        if let Err(status) = cahotic.try_spawn_task(DefaultTask(|| {
            sleep(Duration::from_millis(1000));
            println!("done");
            DefaultOutput(10)
        })) {
            println!("{:?}", status);
        }
    }

    cahotic.join();
}
