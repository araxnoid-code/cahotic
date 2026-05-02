use cahotic::{CahoticBuilder, DefaultJob, DefaultTask, OutputTrait};

enum MyOutput {
    Number(i32),
    Float(f64),
}

impl OutputTrait for MyOutput {}

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_type::<DefaultTask<MyOutput>, DefaultJob<MyOutput>, MyOutput>()
        .build()
        .unwrap();

    cahotic.spawn_task(DefaultTask(|| MyOutput::Number(100)));
    cahotic.spawn_task(DefaultTask(|| MyOutput::Float(32.0)));

    cahotic.join();
}
