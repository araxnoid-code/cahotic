use std::{thread::sleep, time::Duration};

use crate::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

enum MyOutput {
    _Result(i32),
    None,
}
impl OutputTrait for MyOutput {}

enum MyTask {
    Task(fn() -> MyOutput),
    _Schedule(fn(scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput),
}

impl TaskTrait<MyOutput> for MyTask {
    fn execute(&self) -> MyOutput {
        match self {
            MyTask::Task(f) => f(),
            MyTask::_Schedule(_) => MyOutput::None,
        }
    }
}

impl SchedulerTrait<MyOutput> for MyTask {
    fn execute(&self, scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::_Schedule(f) => f(scheduler_vec),
        }
    }
}

#[test]
fn spawn_task() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    // testing 1
    // check spawn 1 task
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));

    // testing 2
    // check spawn 3 task
    // // 1
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));

    // // 2
    for _ in 0..64 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    for _ in 0..64 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    for _ in 0..64 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    // testing 3
    // check spawn 3 task yang delay
    // // 1
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

    // // 2
    for _ in 0..64 {
        cahotic.spawn_task(MyTask::Task(|| {
            sleep(Duration::from_millis(100));
            MyOutput::None
        }));
    }

    for _ in 0..64 {
        cahotic.spawn_task(MyTask::Task(|| {
            sleep(Duration::from_millis(100));
            MyOutput::None
        }));
    }

    // testing 4
    for _ in 0..64 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    for _ in 0..64 * 8 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    for _ in 0..64 * 16 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    for _ in 0..64 * 32 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    for _ in 0..64 * 64 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    for _ in 0..64 * 128 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    cahotic.join();
}
