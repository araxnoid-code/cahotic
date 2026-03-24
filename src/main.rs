use cahotic::{Cahotic, Schedule, ScheduleUnit};
use std::{
    sync::atomic::{AtomicIsize, AtomicU32},
    u64,
};

use std::{thread::sleep, time::Duration};

use cahotic::{OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

#[derive(Debug)]
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
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 10, 32>::init();

    for i in 0..100 {
        let empty_bitmap = cahotic
            .list_core
            .packet_core
            .empty_bitmap
            .load(std::sync::atomic::Ordering::Acquire);
        println!("{:064b}", empty_bitmap);

        let mut poll1 = ScheduleUnit::create_task(MyTask::Task(|| {
            sleep(Duration::from_millis(1000));
            MyOutput::Result(10)
        }));

        cahotic.submit_packet();

        let mut poll2 = ScheduleUnit::create_task(MyTask::Task(|| {
            sleep(Duration::from_millis(2000));
            MyOutput::Result(20)
        }));

        cahotic.submit_packet();

        let mut poll3 = ScheduleUnit::create_schedule(MyTask::Schedule(|schedule_vec| {
            sleep(Duration::from_millis(1500));

            if let (Some(MyOutput::Result(poll1)), Some(MyOutput::Result(poll2))) =
                (schedule_vec.get(0), schedule_vec.get(1))
            {
                MyOutput::Result((*poll1 + *poll2) * 2)
            } else {
                MyOutput::None
            }
        }));
        poll3.after(&mut poll1).unwrap();
        poll3.after(&mut poll2).unwrap();

        let mut poll4 = ScheduleUnit::create_schedule(MyTask::Schedule(|schedule_vec| {
            sleep(Duration::from_millis(1500));

            if let (Some(MyOutput::Result(poll1)), Some(MyOutput::Result(poll2))) =
                (schedule_vec.get(0), schedule_vec.get(1))
            {
                MyOutput::Result((*poll1 + *poll2) * 2)
            } else {
                MyOutput::None
            }
        }));
        poll4.after(&mut poll3).unwrap();
        poll4.after(&mut poll1).unwrap();
        cahotic.schedule_exec(poll3);
        cahotic.schedule_exec(poll1);
        cahotic.schedule_exec(poll2);
        cahotic.schedule_exec(poll4);
    }
    cahotic.submit_packet();

    println!("done spawning");
    loop {
        let empty_bitmap = cahotic
            .list_core
            .packet_core
            .empty_bitmap
            .load(std::sync::atomic::Ordering::Acquire);
        println!("{:064b}", empty_bitmap);

        sleep(Duration::from_millis(1000));

        if empty_bitmap == u64::MAX {
            break;
        }
    }

    cahotic.join();
}
