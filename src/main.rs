use std::{
    sync::atomic::{AtomicU64, Ordering},
    thread::sleep,
    time::Duration,
};

use cahotic::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

enum MyOutput {
    _Result(i32),
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

    let mut poll1 = cahotic
        .list_core
        .packet_core
        .create_task_schedule(MyTask::Task(|| {
            println!("done 1");
            MyOutput::None
        }));

    let mut poll2 = cahotic
        .list_core
        .packet_core
        .create_schedule(MyTask::Schedule(|value| {
            println!("done 2");
            MyOutput::None
        }));

    // let poll3 = cahotic
    //     .list_core
    //     .packet_core
    //     .create_schedule(MyTask::Schedule(|_| {
    //         println!("done 3");
    //         MyOutput::None
    //     }));

    // let poll4 = cahotic
    //     .list_core
    //     .packet_core
    //     .create_schedule(MyTask::Schedule(|_| {
    //         println!("done 4");
    //         MyOutput::None
    //     }));

    cahotic
        .list_core
        .packet_core
        .schedule_after(&mut poll2, &mut poll1)
        .unwrap();

    cahotic.list_core.schedule_wait_exec(poll2);
    cahotic.list_core.schedule_wait_exec(poll1);

    cahotic.submit_packet();

    // sleep(Duration::from_millis(2000));
    let schedule_bitmap = cahotic
        .list_core
        .packet_core
        .allo_schedule_bitmap
        .load(Ordering::Acquire);
    println!("{:064b}", schedule_bitmap);

    cahotic.join();

    // for i in 0..2000 {
    //     // spawn task
    //     cahotic.spawn_task(MyTask::Task(|| {
    //         sleep(Duration::from_millis(100));
    //         MyOutput::None
    //     }));

    //     // submit packet
    // }
    // cahotic.submit_packet();

    // cahotic.join();
}
