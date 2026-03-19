use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, OutputTrait, Scheduler, SchedulerTrait, SchedulerVec, TaskTrait};

#[derive(Debug)]
enum MyOutput {
    Result(i32),
    None,
}
impl OutputTrait for MyOutput {}

enum MyTask {
    Task(fn() -> MyOutput),
    Schedule(fn(scheduler_vec: SchedulerVec<MyOutput>) -> MyOutput),
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
    fn execute(&self, scheduler_vec: SchedulerVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::Schedule(f) => f(scheduler_vec),
        }
    }
}

fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();
    let running = true;

    while running {
        let poll1 = cahotic.spawn_task(MyTask::Task(|| {
            sleep(Duration::from_millis(500));
            println!("done!");
            MyOutput::Result(10)
        }));

        let poll2 = cahotic.spawn_task(MyTask::Task(|| {
            // sleep(Duration::from_millis(2000));
            println!("done!");
            MyOutput::Result(20)
        }));

        cahotic.swap_drop_arena();
    }

    cahotic.join();
}
