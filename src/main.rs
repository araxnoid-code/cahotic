use cahotic::{Cahotic, Scheduler};
use std::{
    sync::atomic::{AtomicIsize, AtomicU32},
    u64,
};

use std::{thread::sleep, time::Duration};

use cahotic::{OutputTrait, SchedulerTrait, SchedulerVec, TaskTrait};

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
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 10, 32>::init();

    let poll1 = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        MyOutput::Result(10)
    }));

    cahotic.submit_packet();

    let poll2 = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(2000));
        MyOutput::Result(20)
    }));

    cahotic.submit_packet();

    let mut scheduler = Scheduler::init(MyTask::Schedule(|scheduler_vec| {
        sleep(Duration::from_millis(1500));

        if let (Some(MyOutput::Result(poll1)), Some(MyOutput::Result(poll2))) =
            (scheduler_vec.get(0), scheduler_vec.get(1))
        {
            MyOutput::Result((*poll1 + *poll2) * 2)
        } else {
            MyOutput::None
        }
    }));

    cahotic.submit_packet();

    scheduler.after(&poll1);
    scheduler.after(&poll2);
    let poll3 = cahotic.scheduler_exec(scheduler);

    cahotic.submit_packet();

    // let mut scheduler = Scheduler::init(MyTask::Schedule(|scheduler_vec| {
    //     sleep(Duration::from_millis(250));
    //     let poll3 = scheduler_vec.get(0).unwrap();
    //     println!("{:?}", poll3);
    //     MyOutput::None
    // }));

    // scheduler.after(&poll3);
    // cahotic.scheduler_exec(scheduler);

    // cahotic.submit_packet();

    cahotic.join();
}
