use std::{thread::sleep, time::Duration};

use cahotic::{
    Cahotic, OutputTrait, PoolOutput, TaskDependenciesTrait, TaskTrait, TaskWithDependenciesTrait,
};

#[derive(Debug)]
enum MyOutput {
    Number(i32),
    None,
}

impl MyOutput {
    fn get_number(&self) -> Option<i32> {
        if let MyOutput::Number(num) = self {
            Some(*num)
        } else {
            None
        }
    }
}

impl OutputTrait for MyOutput {}

enum MyTask {
    Exec(fn() -> MyOutput),
    ExecWithDependencies(fn(dependencies: &'static Vec<PoolOutput<MyOutput>>) -> MyOutput),
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
    fn execute(&self, dependencies: &'static Vec<PoolOutput<MyOutput>>) -> MyOutput {
        match self {
            MyTask::Exec(f) => f(),
            MyTask::ExecWithDependencies(f) => f(dependencies),
        }
    }
}

impl TaskDependenciesTrait<MyTask, MyOutput> for Vec<MyTask> {
    fn task_list(self) -> Vec<MyTask> {
        self
    }
}

fn main() {
    let thread_pool: Cahotic<MyTask, MyTask, MyOutput, 8> = Cahotic::init();

    for i in 0..1000 {
        thread_pool.spawn_task(MyTask::Exec(|| MyOutput::Number(10)));
    }

    thread_pool.join();
}
