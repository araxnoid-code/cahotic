use std::{thread::sleep, time::Duration};

use crate::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

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

#[test]
fn spawn_task() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    // testing 1
    // check spawn 1 task
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    cahotic.submit_packet();

    // testing 2
    // check spawn 3 task
    // // pada satu packet
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    cahotic.submit_packet();
    // // berbeda packet
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    cahotic.submit_packet();
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    cahotic.submit_packet();
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    cahotic.submit_packet();

    // testing 3
    // check spawn 3 task yang delay
    // // pada satu packet
    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        MyOutput::None
    }));
    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        MyOutput::None
    }));
    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        MyOutput::None
    }));
    cahotic.submit_packet();
    // // berbeda packet
    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        MyOutput::None
    }));
    cahotic.submit_packet();
    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        MyOutput::None
    }));
    cahotic.submit_packet();
    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        MyOutput::None
    }));
    cahotic.submit_packet();

    // testing 4
    // // spawn task melebihi kapasitas packet size
    for i in 0..64 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }
    cahotic.submit_packet();

    for i in 0..64 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
        cahotic.submit_packet();
    }
    cahotic.submit_packet();

    cahotic.join();
}
