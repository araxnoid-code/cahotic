use std::{thread::sleep, time::Duration};

use cahotic::{
    Cahotic, OutputTrait, PoolWait, TaskDependenciesTrait, TaskTrait, TaskWithDependenciesTrait,
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
    ExecWithDependencies(fn(dependencies: &'static Vec<PoolWait<MyOutput>>) -> MyOutput),
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
    fn execute(&self, dependencies: &'static Vec<PoolWait<MyOutput>>) -> MyOutput {
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

    for i in 0..20 {
        let list_of_task = vec![
            MyTask::Exec(|| {
                sleep(Duration::from_millis(1000));
                println!("task 1A done");
                MyOutput::Number(10)
            }),
            MyTask::Exec(|| {
                sleep(Duration::from_millis(2000));
                println!("task 2A done");
                MyOutput::Number(20)
            }),
            MyTask::Exec(|| {
                sleep(Duration::from_millis(2000));
                println!("task 3A done");
                MyOutput::Number(20)
            }),
        ];

        let other_list_of_task = vec![
            MyTask::Exec(|| {
                sleep(Duration::from_millis(1000));
                println!("task 1B done");
                MyOutput::Number(10)
            }),
            MyTask::Exec(|| {
                sleep(Duration::from_millis(2000));
                println!("task 2B done");
                MyOutput::Number(20)
            }),
        ];

        let dependencies = thread_pool.spwan_dependencies(list_of_task);
        let other_dependencies = thread_pool.spwan_dependencies(other_list_of_task);

        let task = thread_pool.spawn_task_with_dependencies(
            MyTask::ExecWithDependencies(|dependencies| {
                sleep(Duration::from_millis(500));

                let task_1 = dependencies[0].get().unwrap().get_number().unwrap();
                let task_2 = dependencies[1].get().unwrap().get_number().unwrap();

                println!(
                    "task 4A done with {} from task 1A and {} from task 2A",
                    task_1, task_2
                );
                MyOutput::Number(task_1 + task_2)
            }),
            &dependencies,
        );

        let task = thread_pool.spawn_task_with_dependencies(
            MyTask::ExecWithDependencies(|dependencies| {
                sleep(Duration::from_millis(500));

                let task_1 = dependencies[0].get().unwrap().get_number().unwrap();
                let task_2 = dependencies[1].get().unwrap().get_number().unwrap();

                println!(
                    "task 3B done with {} from task 1B and {} from task 2B",
                    task_1, task_2
                );
                MyOutput::Number(task_1 + task_2)
            }),
            &other_dependencies,
        );

        let task = thread_pool.spawn_task_with_dependencies(
            MyTask::ExecWithDependencies(|dependencies| {
                sleep(Duration::from_millis(250));

                let task_1 = dependencies[0].get().unwrap().get_number().unwrap();
                let task_3 = dependencies[2].get().unwrap().get_number().unwrap();

                println!(
                    "task 5 done with {} from task 1 and {} from task 2",
                    task_1, task_3
                );
                MyOutput::Number(task_1 + task_3)
            }),
            &dependencies,
        );
    }

    thread_pool.join();
}
