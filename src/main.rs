use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultSchedule, DefaultTask, OutputTrait};

struct MyOutput(&'static str);
impl OutputTrait for MyOutput {}

fn main() {
    let cahotic = CahoticBuilder::default::<i32>()
        .set_type::<DefaultTask<MyOutput>, DefaultSchedule<MyOutput>, MyOutput>()
        .build()
        .unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        MyOutput("ok")
    }));

    cahotic.join();
}
