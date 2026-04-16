# Version/0.3.0
- Added `CahoticBuilder` for cahotic initialization which allows fast initialization with default values and clear changes.
1. setting ring buffer size and workers
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<Option<i32>>()
        .set_ring_buffer_size::<4096>()
        .set_workers::<8>()
        .build()
        .unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        DefaultOutput(None)
    }));

    cahotic.join();
}
```

2. Setting Task and Shcedule
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, ScheduleVec, SchedulerTrait, TaskTrait};

// Task
struct MyTask {
    f: fn(usize) -> DefaultOutput<usize>,
    input: usize,
}

impl TaskTrait<DefaultOutput<usize>> for MyTask {
    fn execute(&self) -> DefaultOutput<usize> {
        (self.f)(self.input)
    }
}

// Schedule
struct MySchedule(fn(scheduler_vec: ScheduleVec<DefaultOutput<usize>>) -> DefaultOutput<usize>);

impl SchedulerTrait<DefaultOutput<usize>> for MySchedule {
    fn execute(&self, scheduler_vec: ScheduleVec<DefaultOutput<usize>>) -> DefaultOutput<usize> {
        (self.0)(scheduler_vec)
    }
}

fn main() {
    let cahotic = CahoticBuilder::default()
        .set_ring_buffer_size::<4096>()
        .set_workers::<8>()
        // setting Task
        .set_task_type::<MyTask>()
        // setting Schedule
        .set_schedule_type::<MySchedule>()
        .build()
        .unwrap();

    // Task
    for i in 0..5 {
        cahotic.spawn_task(MyTask {
            f: |i| {
                sleep(Duration::from_millis(1000));
                println!("task {} done", i);
                DefaultOutput(0)
            },
            input: i,
        });
    }

    // Schedule
    let mut poll_1 = cahotic.scheduling_create_initial(MyTask {
        f: |i| {
            sleep(Duration::from_millis(1000));
            println!("schedule {} done", i);
            DefaultOutput(0)
        },
        input: 1,
    });

    let mut poll_2 = cahotic.scheduling_create_schedule(MySchedule(|shcedule_vec| {
        println!("schedule 2 done");
        DefaultOutput(10)
    }));

    cahotic.schedule_after(&mut poll_2, &mut poll_1).unwrap();

    cahotic.schedule_exec(poll_2);
    cahotic.schedule_exec(poll_1);

    cahotic.join();
}
```

3. Setting Output
```rust
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
```


- Added `Cahotic::try_spawn_task(&self, F)` method to spawn tasks which will return Err(TryEnqueueStatus) if the ring_buffer is full, this will not cause blocking like `Cahotic::spawn_task(&self, F)`.
