use std::{
    sync::atomic::{AtomicU64, Ordering},
    thread::sleep,
    time::Duration,
};

use cahotic::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

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
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    for i in 0..2042 {
        let mut poll1 = cahotic.scheduling_create_task(MyTask::Task(|| {
            sleep(Duration::from_millis(500));
            // println!("1");
            MyOutput::Result(10)
        }));

        let mut poll2 = cahotic.scheduling_create_task(MyTask::Task(|| {
            sleep(Duration::from_millis(250));
            // println!("2");
            MyOutput::Result(20)
        }));

        let mut poll3 = cahotic.scheduling_create_schedule(MyTask::Schedule(|vec| {
            sleep(Duration::from_millis(100));
            // println!("3");
            let value1 = vec.get(0);
            let value2 = vec.get(1);
            MyOutput::Result(30)
        }));

        cahotic.schedule_after(&mut poll3, &mut poll1).unwrap();
        cahotic.schedule_after(&mut poll3, &mut poll2).unwrap();

        let mut poll4 = cahotic.scheduling_create_schedule(MyTask::Schedule(|vec| {
            sleep(Duration::from_millis(100));
            // println!("3");
            let value1 = vec.get(0);
            let value2 = vec.get(1);
            let value3 = vec.get(2);
            MyOutput::Result(30)
        }));
        cahotic.schedule_after(&mut poll4, &mut poll1).unwrap();
        cahotic.schedule_after(&mut poll4, &mut poll2).unwrap();
        cahotic.schedule_after(&mut poll4, &mut poll3).unwrap();

        cahotic.schedule_exec(poll3);
        cahotic.schedule_exec(poll2);
        cahotic.schedule_exec(poll1);
        cahotic.schedule_exec(poll4);
    }

    cahotic.submit_packet();

    // println!("done");
    // loop {
    //     println!("=====================");
    //     let bitmap = cahotic
    //         .task_core
    //         .packet_core
    //         .allo_schedule_bitmap
    //         .load(Ordering::Acquire);
    //     println!("{:064b}", bitmap);

    //     let bitmap = cahotic
    //         .task_core
    //         .packet_core
    //         .poll_schedule_bitmap
    //         .load(Ordering::Acquire);
    //     println!("{:064b}", bitmap);

    // //     sleep(Duration::from_millis(500));
    // }

    cahotic.join();
}
