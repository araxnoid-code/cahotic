use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_workers::<16>()
        .set_ring_buffer_size::<4096>()
        .build();

    cahotic.spawn_task(DefaultTask(|| {
        println!("done");
        DefaultOutput(10)
    }));

    cahotic.join();
}
