use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

#[derive(Debug)]
enum MyOutput {
    Result(i32),
    None,
}
impl OutputTrait for MyOutput {}

enum MyTask {
    Task((usize, fn() -> MyOutput)),
    Schedule(fn(scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput),
}

impl TaskTrait<MyOutput> for MyTask {
    fn execute(&self) -> MyOutput {
        match self {
            MyTask::Task((idx, f)) => {
                // println!("{idx} done");
                f()
            }
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
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 10, 16>::init();

    for i in 0..63 {
        cahotic.spawn_task(MyTask::Task((i, || MyOutput::None)));
    }

    let mut poll1 = cahotic.scheduling_create_initial(MyTask::Task((0, || {
        sleep(Duration::from_millis(1000));
        println!("1 done");
        MyOutput::Result(10)
    })));

    let mut poll2 = cahotic.scheduling_create_schedule(MyTask::Schedule(|vec| {
        sleep(Duration::from_millis(2000));
        let value = vec.get(0);
        println!("2 done with poll1 value {:?}", value);
        MyOutput::Result(20)
    }));

    cahotic.schedule_after(&mut poll2, &mut poll1).unwrap();

    // let mut poll3 = cahotic.scheduling_create_schedule(MyTask::Schedule(|vec| {
    //     // sleep(Duration::from_millis(2000));
    //     let value = vec.get(0);
    //     println!("3 done with poll2 value {:?}", value);
    //     MyOutput::Result(20)
    // }));
    // cahotic.schedule_after(&mut poll3, &mut poll2).unwrap();

    cahotic.schedule_exec(poll1);
    cahotic.schedule_exec(poll2);
    // cahotic.schedule_exec(poll3);

    // println!("{:?}", poll1.block());

    // for i in 0..64 * 100 {
    // let bitmap = cahotic.get_quota_bitmap();
    // println!("{:064b} | {}", bitmap, i);

    // let poll = cahotic.spawn_task_update(MyTask::Task(|| {
    //     sleep(Duration::from_millis(50));
    //     // println!("done!");
    //     MyOutput::None
    // }));
    // }

    // println!("done");

    // loop {
    //     let bitmap = cahotic.get_quota_bitmap();
    //     println!("{:064b}", bitmap);
    //     sleep(Duration::from_millis(10));

    //     if bitmap == u64::MAX {
    //         break;
    //     }
    // }
    // println!("==============> head: {}", head);

    cahotic.join();
}
