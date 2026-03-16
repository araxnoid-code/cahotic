use std::{thread::sleep, time::Duration};

use cahotic::{
    Cahotic, OutputTrait, PollWaiting, TaskDependenciesTrait, TaskTrait, TaskWithDependenciesTrait,
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
    ExecWithDependencies(fn(dependencies: &'static Vec<PollWaiting<MyOutput>>) -> MyOutput),
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
    fn execute(&self, dependencies: &'static Vec<PollWaiting<MyOutput>>) -> MyOutput {
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
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    for i in 0..1 {
        let list_task = vec![
            MyTask::Exec(|| {
                sleep(Duration::from_millis(550));
                // println!("task 1 done");
                MyOutput::Number(10)
            }),
            MyTask::Exec(|| {
                // println!("task 2 done");
                MyOutput::Number(20)
            }),
            MyTask::Exec(|| {
                sleep(Duration::from_millis(1250));
                // println!("task 3 done");
                MyOutput::Number(30)
            }),
        ];

        let dependencies = cahotic.spwan_dependencies(list_task);

        let task_4 = cahotic.spawn_task_with_dependencies(
            MyTask::ExecWithDependencies(|dependencies| {
                let task_1 = dependencies[0].get().unwrap().get_number().unwrap();
                let task_2 = dependencies[1].get().unwrap().get_number().unwrap();
                let task_3 = dependencies[2].get().unwrap().get_number().unwrap();

                // println!("task 4 done");
                MyOutput::Number(task_1 + task_2 + task_3)
            }),
            &dependencies,
        );

        let task_5 = cahotic.spawn_task_with_dependencies(
            MyTask::ExecWithDependencies(|dependencies| {
                // sleep(Duration::from_millis(2000));
                let task_1 = dependencies[0].get().unwrap().get_number().unwrap();
                let task_3 = dependencies[2].get().unwrap().get_number().unwrap();

                // println!("task 5 done");
                MyOutput::Number(task_1 + task_3)
            }),
            &dependencies,
        );

        let task_6 = cahotic.spawn_task_with_dependencies(
            MyTask::ExecWithDependencies(|dependencies| {
                // sleep(Duration::from_millis(3000));
                let task_1 = dependencies[0].get().unwrap().get_number().unwrap();
                let task_2 = dependencies[1].get().unwrap().get_number().unwrap();

                // println!("task 6 done");
                MyOutput::Number(task_1 + task_2)
            }),
            &dependencies,
        );

        // bahaya!, dependencies bisa saja akan dihapus terlebih dahulu sebelum task 4, 5, 6 selesai
        // cahotic.drop_poll(task_4);
        // cahotic.drop_poll(task_5);
        // cahotic.drop_poll(task_6);
        // cahotic.drop_dependencies(dependencies);

        // chaining drop
        // cahotic.drop_after(task_4, &task_5);
        // pada task 4 di drop setelah task 5 selesai, task 5 tidak akan di drop karena memiliki after_drop_counter > 0 maka task 5 bisa di drop saat task 4 selesai di drop terlebih dahulu
        // cahotic.drop_after(task_5, &task_6);
        // // masih sama dengan konsep sebelumnya
        // cahotic.drop_after(dependencies, &task_6);
        // cahotic.drop_poll(task_6);
    }

    cahotic.join();
}
