use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, OutputTrait, PoolTask, TaskTrait, TaskWithDependenciesTrait};

#[derive(Debug)]
enum MyOutput {
    Number(i32),
    String(String),
    None,
}

impl OutputTrait for MyOutput {}

enum MyTask {
    Exec(fn() -> MyOutput),
    ExecWithDependencies(fn(dependencies: &'static Vec<PoolTask<MyOutput>>) -> MyOutput),
}

impl TaskTrait<MyOutput> for MyTask {
    fn execute(&self) -> MyOutput {
        match self {
            MyTask::Exec(f) => f(),
            _ => MyOutput::None,
        }
    }
}

impl TaskWithDependenciesTrait<MyOutput> for MyTask {
    fn execute(&self, dependencies: &'static Vec<PoolTask<MyOutput>>) -> MyOutput {
        match self {
            MyTask::Exec(f) => f(),
            _ => MyOutput::None,
        }
    }
}

fn main() {
    let thread_pool: Cahotic<MyTask, MyTask, MyOutput, 8> = Cahotic::init();

    for i in 0..32 {
        let pool = thread_pool.spawn_task(MyTask::Exec(|| {
            sleep(Duration::from_millis(1000));
            MyOutput::Number(10)
        }));
    }

    thread_pool.join();
}
