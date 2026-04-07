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
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 16>::init();

    // for i in 0..100 {
    //     cahotic.spawn_task(MyTask::Task(|| {
    //         sleep(Duration::from_millis(1000));
    //         MyOutput::None
    //     }));
    // }

    // real	0m7.015s
    // user	0m7.092s
    // sys	0m0.266s

    // real	0m7.017s
    // user	0m7.147s
    // sys	0m0.269s

    // real	0m7.016s
    // user	0m7.145s
    // sys	0m0.270s

    // for i in 0..1000 {
    //     cahotic.spawn_task(MyTask::Task(|| {
    //         sleep(Duration::from_millis(100));
    //         MyOutput::None
    //     }));
    // }
    // real	0m6.323s
    // user	0m6.321s
    // sys	0m0.030s

    // real	0m6.321s
    // user	0m6.316s
    // sys	0m0.032s

    // real	0m6.321s
    // user	0m6.309s
    // sys	0m0.035s

    // for i in 0..4096 * 2 {
    //     cahotic.spawn_task(MyTask::Task(|| {
    //         sleep(Duration::from_millis(100));
    //         MyOutput::None
    //     }));
    // }

    // real	0m51.294s
    // user	0m51.203s
    // sys	0m0.061s

    // real	0m51.292s
    // user	0m51.189s
    // sys	0m0.069s

    // real	0m51.288s
    // user	0m51.204s
    // sys	0m0.060s

    cahotic.join();
}
