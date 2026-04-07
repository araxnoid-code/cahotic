<div align="center">
    <h1>cahotic</h1>
    <b><p>Thread Pool Management</p></b>
    <p>⚙️ under development ⚙️</p>
    <b>
        <p>Version / 0.1.0</p>
    </b>
</div>

## About
`cahotic`, thread pool management written in rust.

## Vesrion
what's new with: [version/0.1.0](https://github.com/araxnoid-code/cahotic/blob/version/0.1.0/version.md)

## Guide 
explanation of main features (English and Indonesian available): [guide.md](https://github.com/araxnoid-code/cahotic/blob/version/0.1.0/guide/guide.md)

## Starting
### Installation
Run the following Cargo command in your project directory:
```sh
cargo add cahotic
```
Or add the following line to your Cargo.toml:
```toml
cahotic = "0.1.0"
```

### Code
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

enum MyOutput {
    Result(i32),
    None,
}
impl OutputTrait for MyOutput {}

enum MyTask {
    Task(fn() -> MyOutput),
    Schedule(fn(scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput),
}

impl TaskTrait<MyOutput> for MyTask {
    fn execute(&self) -> MyOutput {
        match self {
            MyTask::Task(f) => f(),
            MyTask::Schedule(_) => MyOutput::None,
        }
    }
}

impl SchedulerTrait<MyOutput> for MyTask {
    fn execute(&self, scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::Schedule(f) => f(scheduler_vec),
        }
    }
}

fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));

    cahotic.submit_packet();

    cahotic.join();
}
```
