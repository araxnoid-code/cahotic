use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultSchedule, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default().build().unwrap();

    // poll masih akan dieksekusi namun memiliki cost untuk penanganannya, gunakan initial schedule.
    let poll = cahotic.scheduling_create_schedule(DefaultSchedule(|_| {
        println!("task done");
        DefaultOutput(10)
    }));

    cahotic.schedule_exec(poll);

    cahotic.join();
}
