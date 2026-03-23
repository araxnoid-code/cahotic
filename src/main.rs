use cahotic::{Cahotic, Scheduler};
use std::{
    sync::atomic::{AtomicIsize, AtomicU32},
    u64,
};

use std::{thread::sleep, time::Duration};

use cahotic::{OutputTrait, SchedulerTrait, SchedulerVec, TaskTrait};

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
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 10, 32>::init();

    for i in 0..32 {
        let poll1 = cahotic.spawn_task(MyTask::Task(|| {
            sleep(Duration::from_millis(1000));
            MyOutput::None
        }));

        let poll2 = cahotic.spawn_task(MyTask::Task(|| {
            sleep(Duration::from_millis(500));
            MyOutput::None
        }));

        let mut scheduler = Scheduler::init(MyTask::Schedule(|_| MyOutput::None));

        scheduler.after(&poll1);
        scheduler.after(&poll2);
        cahotic.scheduler_exec(scheduler);
    }

    cahotic.submit_packet();

    cahotic.join();
}
