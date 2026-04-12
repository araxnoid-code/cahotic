use std::{thread::sleep, time::Duration};

use crate::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

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

#[test]
fn schedule_1() {
    let cahotic: Cahotic<MyTask, MyTask, MyOutput, 4> = Cahotic::init();

    let mut poll_1 = cahotic.scheduling_create_initial(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        MyOutput::Result(10)
    }));
    let mut poll_2 = cahotic.scheduling_create_schedule(MyTask::Schedule(|schedule_vec| {
        if let Some(MyOutput::Result(x)) = schedule_vec.get(0) {
            assert_eq!(*x, 10);
        } else {
            panic!("Schedule Testing Error")
        }

        MyOutput::None
    }));

    cahotic.schedule_after(&mut poll_2, &mut poll_1).unwrap();

    cahotic.schedule_exec(poll_2);
    cahotic.schedule_exec(poll_1);

    cahotic.join();
}

#[test]
fn schedule_2() {
    let cahotic: Cahotic<MyTask, MyTask, MyOutput, 4> = Cahotic::init();

    let poll = cahotic.scheduling_create_schedule(MyTask::Schedule(|_| MyOutput::None));
    cahotic.schedule_exec(poll);

    cahotic.join();
}

#[test]
fn schedule_3() {
    let cahotic: Cahotic<MyTask, MyTask, MyOutput, 4> = Cahotic::init();

    // Testing 1
    for _ in 0..62 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    let mut poll_1 = cahotic.scheduling_create_initial(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        MyOutput::Result(10)
    }));

    let mut poll_2 = cahotic.scheduling_create_schedule(MyTask::Schedule(|schedule_vec| {
        if let Some(MyOutput::Result(x)) = schedule_vec.get(0) {
            assert_eq!(*x, 10);
        } else {
            panic!("Schedule Testing Error")
        }

        MyOutput::None
    }));

    cahotic.schedule_after(&mut poll_2, &mut poll_1).unwrap();

    cahotic.schedule_exec(poll_2);
    cahotic.schedule_exec(poll_1);

    // Testing 2
    for _ in 0..63 {
        cahotic.spawn_task(MyTask::Task(|| MyOutput::None));
    }

    let mut poll_1 = cahotic.scheduling_create_initial(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        MyOutput::Result(10)
    }));

    let mut poll_2 = cahotic.scheduling_create_schedule(MyTask::Schedule(|schedule_vec| {
        if let Some(MyOutput::Result(x)) = schedule_vec.get(0) {
            assert_eq!(*x, 10);
        } else {
            panic!("Schedule Testing Error")
        }

        MyOutput::None
    }));

    cahotic.schedule_after(&mut poll_2, &mut poll_1).unwrap();

    cahotic.schedule_exec(poll_2);
    cahotic.schedule_exec(poll_1);

    cahotic.join();
}

#[test]
fn schedule_4() {
    let cahotic: Cahotic<MyTask, MyTask, MyOutput, 4> = Cahotic::init();

    let mut poll_1 = cahotic.scheduling_create_initial(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        MyOutput::Result(10)
    }));

    let mut poll_2 = cahotic.scheduling_create_initial(MyTask::Task(|| MyOutput::Result(20)));

    let mut poll_3 = cahotic.scheduling_create_schedule(MyTask::Schedule(|schedule_vec| {
        if let (Some(MyOutput::Result(x)), Some(MyOutput::Result(y))) =
            (schedule_vec.get(0), schedule_vec.get(1))
        {
            assert_eq!(*x, 10);
            assert_eq!(*y, 20);
        } else {
            panic!("Schedule Testing Error")
        }

        MyOutput::None
    }));

    cahotic.schedule_after(&mut poll_3, &mut poll_1).unwrap();
    cahotic.schedule_after(&mut poll_3, &mut poll_2).unwrap();

    cahotic.schedule_exec(poll_3);
    cahotic.schedule_exec(poll_2);
    cahotic.schedule_exec(poll_1);

    cahotic.join();
}

#[test]
fn schedule_5() {
    let cahotic: Cahotic<MyTask, MyTask, MyOutput, 4> = Cahotic::init();

    let mut poll_1 = cahotic.scheduling_create_initial(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        println!("done 1");
        MyOutput::Result(10)
    }));

    let mut poll_2 = cahotic.scheduling_create_schedule(MyTask::Schedule(|schedule_vec| {
        if let Some(MyOutput::Result(x)) = schedule_vec.get(0) {
            assert_eq!(*x, 10);
        } else {
            panic!("Schedule Testing Error")
        }
        println!("done 2");

        MyOutput::Result(20)
    }));

    cahotic.schedule_after(&mut poll_2, &mut poll_1).unwrap();

    let mut poll_3 = cahotic.scheduling_create_schedule(MyTask::Schedule(|schedule_vec| {
        if let Some(MyOutput::Result(x)) = schedule_vec.get(0) {
            assert_eq!(*x, 20);
        } else {
            panic!("Schedule Testing Error")
        }
        println!("done 3");

        MyOutput::None
    }));

    cahotic.schedule_after(&mut poll_3, &mut poll_2).unwrap();

    cahotic.schedule_exec(poll_1);
    sleep(Duration::from_millis(1000));
    sleep(Duration::from_millis(1000));
    cahotic.schedule_exec(poll_2);
    sleep(Duration::from_millis(1000));
    cahotic.schedule_exec(poll_3);

    cahotic.join();
}
