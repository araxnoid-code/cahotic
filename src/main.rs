use std::{thread::sleep, time::Duration};

use cahotic::{
    Cahotic, OutputTrait, PollWaiting, Scheduler, SchedulerTrait, SchedulerVec, TaskTrait,
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
    Schdule(fn(dependencies: SchedulerVec<MyOutput>) -> MyOutput),
}

impl TaskTrait<MyOutput> for MyTask {
    fn execute(&self) -> MyOutput {
        match self {
            MyTask::Exec(f) => f(),
            _ => MyOutput::None,
        }
    }
}

impl SchedulerTrait<MyOutput> for MyTask {
    fn execute(&self, dependencies: SchedulerVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Exec(f) => f(),
            MyTask::Schdule(f) => f(dependencies),
        }
    }
}

fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    for i in 0..100 {
        let poll1 = cahotic.spawn_task(MyTask::Exec(|| MyOutput::Number(10)));

        let poll2 = cahotic.spawn_task(MyTask::Exec(|| MyOutput::Number(20)));

        let mut schedule = Scheduler::init(MyTask::Schdule(|schedule_vec| {
            let poll1 = schedule_vec.get(0).unwrap();
            let poll2 = schedule_vec.get(1).unwrap();
            MyOutput::Number(50)
        }));

        schedule.before(&poll1);
        schedule.before(&poll2);

        let poll3 = cahotic.scheduler_exec(schedule);

        let mut schedule = Scheduler::init(MyTask::Schdule(|schedule_vec| {
            let poll1 = schedule_vec.get(0).unwrap();
            let poll2 = schedule_vec.get(1).unwrap();
            MyOutput::Number(40)
        }));

        schedule.before(&poll1);
        schedule.before(&poll3);

        let poll4 = cahotic.scheduler_exec(schedule);

        let mut schedule = Scheduler::init(MyTask::Schdule(|schedule_vec| {
            let poll1 = schedule_vec.get(0).unwrap();
            let poll2 = schedule_vec.get(1).unwrap();
            MyOutput::Number(poll1.get_number().unwrap() + poll2.get_number().unwrap())
        }));

        schedule.before(&poll3);
        schedule.before(&poll4);

        let poll5 = cahotic.scheduler_exec(schedule);

        let mut schedule = Scheduler::init(MyTask::Schdule(|schedule_vec| {
            let poll1 = schedule_vec.get(0).unwrap();
            MyOutput::None
        }));

        schedule.before(&poll5);

        let poll6 = cahotic.scheduler_exec(schedule);

        cahotic.swap_drop_arena();
    }

    cahotic.join();
}
