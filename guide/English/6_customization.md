# Customization
cahotic has provided DefaultOutput, DefaultTask, and DefaultSchedule for rapid development. However, if there are cases that require customization, cahotic still provides them as in the initial version (version/0.2.1 and below), but in the latest version has added CahoticBuilder which improves readability, therefore it is recommended to use `CahoticBuilder`.

## Default
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultSchedule, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    // task
    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        DefaultOutput(10)
    }));

    // scheduling
    let mut poll_1 = cahotic.scheduling_create_initial(DefaultTask(|| DefaultOutput(20)));
    let mut poll_2 =
        cahotic.scheduling_create_schedule(DefaultSchedule(|schedule_vec| DefaultOutput(20)));
    cahotic.schedule_after(&mut poll_2, &mut poll_1).unwrap();

    cahotic.schedule_exec(poll_2);
    cahotic.schedule_exec(poll_1);

    cahotic.join();
}

```
this is a default feature.
1. for output use `DefaultOutput`
2. for tasks using `DefaultTask`
3. to schedule use `DefaultSchedule`
4. for the ring buffer size is 4096
5. for the number of workers is 4

## customize ring buffer size and number of workers
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<i32>()
        .set_ring_buffer_size::<2048>() // ring buffer size
        .set_workers::<8>() // number of workers
        .build()
        .unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        DefaultOutput(10)
    }));

    cahotic.join();
}
```
To customize the size of the ring buffer, you can use the method
`CahoticBuilder::set_ring_buffer_size<const MAX: usize>(self)`
accepts const generic, in const generic this is where the size is customized. There are 2 rules for determining the size for the ring buffer:
1. size must be a multiple of 64
2. size cannot be <= 0

To customize the number of workers, you can use the method
`CahoticBuilder::set_workers<const W: usize>(self)`
accepts const generic, in this const generic the number of workers is customized.

## customize Task
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, TaskTrait};

struct MyTask {
    f: fn(usize) -> DefaultOutput<usize>,
    input: usize,
}

impl TaskTrait<DefaultOutput<usize>> for MyTask {
    fn execute(&self) -> DefaultOutput<usize> {
        (self.f)(self.input)
    }
}

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_task_type::<MyTask>() // task type
        .build()
        .unwrap();

    for i in 0..10 {
        cahotic.spawn_task(MyTask {
            f: |input| {
                sleep(Duration::from_millis(100));
                println!("task {} done", input);
                DefaultOutput(10)
            },
            input: i,
        });
    }

    cahotic.join();
}
```
To customize a task, a data type that will be used as a task must implement the `TaskTrait` trait and be registered in CahoticBuilder using the `CahoticBuilder::set_task_type<T>(self)` method.

## customize Schedule
```rust
use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask, ScheduleVec, SchedulerTrait, TaskTrait};

struct MySchedule(fn(schedule_vec: ScheduleVec<DefaultOutput<usize>>) -> DefaultOutput<usize>);

impl SchedulerTrait<DefaultOutput<usize>> for MySchedule {
    fn execute(&self, scheduler_vec: ScheduleVec<DefaultOutput<usize>>) -> DefaultOutput<usize> {
        (self.0)(scheduler_vec)
    }
}

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_schedule_type::<MySchedule>() // schedule
        .build()
        .unwrap();

    let mut poll_1 = cahotic.scheduling_create_initial(DefaultTask(|| DefaultOutput(10)));
    let mut poll_2 = cahotic.scheduling_create_initial(DefaultTask(|| DefaultOutput(20)));

    let mut poll_3 = cahotic.scheduling_create_schedule(MySchedule(|vec| {
        let data_1 = vec.get(0).unwrap();
        let data_2 = vec.get(1).unwrap();
        let result = data_1.0 + data_2.0;
        println!("result => {}", result);

        DefaultOutput(result)
    }));

    cahotic.schedule_after(&mut poll_3, &mut poll_1).unwrap();
    cahotic.schedule_after(&mut poll_3, &mut poll_2).unwrap();

    cahotic.schedule_exec(poll_3);
    cahotic.schedule_exec(poll_2);
    cahotic.schedule_exec(poll_1);

    cahotic.join();
}
```
To customize the schedule, a data type that will be used as a schedule must implement the `SchedulerTrait` trait and be registered in CahoticBuilder using the `CahoticBuilder::set_schedule_type<T>(self)` method.

## customize Output
In customizing the output, notation for tasks and schedules is also required.
```rust
use cahotic::{CahoticBuilder, DefaultSchedule, DefaultTask, OutputTrait};

enum MyOutput {
    Number(i32),
    Float(f64),
}

impl OutputTrait for MyOutput {}

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_type::<DefaultTask<MyOutput>, DefaultSchedule<MyOutput>, MyOutput>()
        .build()
        .unwrap();

    cahotic.spawn_task(DefaultTask(|| MyOutput::Number(100)));
    cahotic.spawn_task(DefaultTask(|| MyOutput::Float(32.0)));

    cahotic.join();
}
```
To customize the output, a data type that will be output must implement the `OutputTrait` trait and use the `CahoticBuilder::set_type<TASK, SCHEDULE, OUTPUT>(self)` method.
