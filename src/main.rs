use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    let poll = cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        DefaultOutput(10)
    }));

    // Tidak akan terjadi pemblokiran, tetapi jika polling belum siap, maka akan mengembalikan Option::None
    if let Some(value) = poll.get() {
        println!("{:?}", value.0);
    }

    cahotic.join();
}
