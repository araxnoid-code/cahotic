use std::{
    cell::RefCell,
    sync::atomic::{AtomicU64, Ordering},
    thread::sleep,
    time::Duration,
};

use cahotic::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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

// fn main() {
//     (0..100).into_par_iter().for_each(|_| {
//         fib_recursive(42);
//     });
// }

fn main() {
    unsafe {
        let x: i32 = 10;
        let p1 = x as *const i32 as *mut i32;
        let p2 = x as *const i32 as *mut i32;
        *p1 = 20;
        *p2 = 20;
    }
    // let p2: &mut i32 = &mut x;

    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 16, 32>::init();
    for i in 0..100 {
        cahotic.spawn_task(MyTask::Task(|| {
            fib_recursive(42);
            MyOutput::None
        }));
    }
    cahotic.submit_packet();

    cahotic.join();
}

fn fib_recursive(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib_recursive(n - 1) + fib_recursive(n - 2),
    }
}
