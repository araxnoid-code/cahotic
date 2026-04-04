use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

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

    for i in 0..256 {
        let poll = cahotic.spawn_task_update(MyTask::Task(|| {
            sleep(Duration::from_millis(100));
            // println!("done!");
            MyOutput::None
        }));
    }

    let bitmap = cahotic.get_quota_bitmap();
    println!("{:064b}", bitmap);

    sleep(Duration::from_millis(5000));
    let bitmap = cahotic.get_quota_bitmap();
    println!("{:064b}", bitmap);

    // let bitmap = cahotic.get_quota_bitmap();
    // println!("{:064}", bitmap);
    // println!("==============> head: {}", head);

    cahotic.join();
}
