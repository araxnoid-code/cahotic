use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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
    // let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 16>::init();

    // for i in 0..100 {
    //     cahotic.spawn_task(MyTask::Task(|| {
    //         fib_recursive(42);
    //         MyOutput::None
    //     }));
    // }

    // cahotic.join();

    // cahotic
    // real	0m11.456s
    // user	2m49.444s
    // sys	0m0.306s

    // real	0m11.230s
    // user	2m49.867s
    // sys	0m0.245s

    // real	0m11.320s
    // user	2m50.768s
    // sys	0m0.243s

    // (0..100).into_par_iter().for_each(|_| {
    //     fib_recursive(42);
    // });

    // rayon
    // real	0m10.317s
    // user	2m30.411s
    // sys	0m0.008s

    // real	0m10.185s
    // user	2m30.721s
    // sys	0m0.020s

    // real	0m10.226s
    // user	2m31.113s
    // sys	0m0.031s
}

fn fib_recursive(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib_recursive(n - 1) + fib_recursive(n - 2),
    }
}
