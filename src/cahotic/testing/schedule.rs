use std::{thread::sleep, time::Duration};

use crate::{Cahotic, OutputTrait, Schedule, ScheduleVec, SchedulerTrait, TaskTrait};

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
fn schedule() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    // testing 1
    let mut poll1 = Schedule::create_task(MyTask::Task(|| MyOutput::Result(10)));
    let mut poll2 = Schedule::create_schedule(MyTask::Schedule(|schedule_vec| {
        let poll1_value = schedule_vec
            .get(0)
            .expect("poll2 accessed an invalid index on schedule_vec");

        if let MyOutput::None = poll1_value {
            panic!("Error, poll2 accesses poll1 which has a value of None")
        } else if let MyOutput::Result(value) = poll1_value {
            assert_eq!(*value, 10);
        }

        MyOutput::None
    }));

    poll2.after(&mut poll1).unwrap();

    cahotic.schedule_exec(poll1);
    cahotic.schedule_exec(poll2);
    cahotic.submit_packet();

    // testing 2
    let mut poll1 = Schedule::create_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        MyOutput::Result(20)
    }));
    let mut poll2 = Schedule::create_task(MyTask::Task(|| {
        sleep(Duration::from_millis(250));
        MyOutput::Result(30)
    }));
    let mut poll3 = Schedule::create_schedule(MyTask::Schedule(|schedule_vec| {
        let poll1_value = schedule_vec
            .get(0)
            .expect("poll2 accessed an invalid index on schedule_vec");

        let poll2_value = schedule_vec
            .get(1)
            .expect("poll2 accessed an invalid index on schedule_vec");

        if let (MyOutput::None, MyOutput::None) = (poll1_value, poll2_value) {
            panic!("Error, poll2 accesses poll1 or poll2 which has a value of None")
        } else if let (MyOutput::Result(value1), MyOutput::Result(value2)) =
            (poll1_value, poll2_value)
        {
            assert_eq!(*value1, 20);
            assert_eq!(*value2, 30);
        }

        MyOutput::None
    }));

    poll3.after(&mut poll1).unwrap();
    poll3.after(&mut poll2).unwrap();

    cahotic.schedule_exec(poll1);
    cahotic.schedule_exec(poll3);
    cahotic.schedule_exec(poll2);
    cahotic.submit_packet();

    // testing 3
    let mut poll1 = Schedule::create_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        MyOutput::Result(20)
    }));
    let mut poll2 = Schedule::create_task(MyTask::Task(|| {
        sleep(Duration::from_millis(250));
        MyOutput::Result(30)
    }));
    let mut poll3 = Schedule::create_schedule(MyTask::Schedule(|schedule_vec| {
        let poll1_value = schedule_vec
            .get(0)
            .expect("poll2 accessed an invalid index on schedule_vec");

        let poll2_value = schedule_vec
            .get(1)
            .expect("poll2 accessed an invalid index on schedule_vec");

        if let (MyOutput::None, MyOutput::None) = (poll1_value, poll2_value) {
            panic!("Error, poll2 accesses poll1 or poll2 which has a value of None")
        } else if let (MyOutput::Result(value1), MyOutput::Result(value2)) =
            (poll1_value, poll2_value)
        {
            assert_eq!(*value1, 20);
            assert_eq!(*value2, 30);
        }

        MyOutput::Result(100)
    }));
    poll3.after(&mut poll1).unwrap();
    poll3.after(&mut poll2).unwrap();
    cahotic.schedule_exec(poll1);
    cahotic.schedule_exec(poll3);
    cahotic.schedule_exec(poll2);
    cahotic.submit_packet();

    // testing 4
    let mut poll1 = Schedule::create_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        MyOutput::Result(20)
    }));
    let mut poll2 = Schedule::create_task(MyTask::Task(|| {
        sleep(Duration::from_millis(250));
        MyOutput::Result(30)
    }));

    let mut poll3 = Schedule::create_schedule(MyTask::Schedule(|schedule_vec| {
        let poll1_value = schedule_vec
            .get(0)
            .expect("poll2 accessed an invalid index on schedule_vec");

        let poll2_value = schedule_vec
            .get(1)
            .expect("poll2 accessed an invalid index on schedule_vec");

        if let (MyOutput::None, MyOutput::None) = (poll1_value, poll2_value) {
            panic!("Error, poll2 accesses poll1 or poll2 which has a value of None")
        } else if let (MyOutput::Result(value1), MyOutput::Result(value2)) =
            (poll1_value, poll2_value)
        {
            assert_eq!(*value1, 20);
            assert_eq!(*value2, 30);
        }

        MyOutput::Result(100)
    }));
    poll3.after(&mut poll1).unwrap();
    poll3.after(&mut poll2).unwrap();

    let mut poll4 = Schedule::create_schedule(MyTask::Schedule(|schedule_vec| {
        let poll1_value = schedule_vec
            .get(0)
            .expect("poll2 accessed an invalid index on schedule_vec");

        let poll2_value = schedule_vec
            .get(1)
            .expect("poll2 accessed an invalid index on schedule_vec");

        let poll3_value = schedule_vec
            .get(2)
            .expect("poll2 accessed an invalid index on schedule_vec");

        if let (MyOutput::None, MyOutput::None, MyOutput::None) =
            (poll1_value, poll2_value, poll3_value)
        {
            panic!("Error, poll2 accesses poll1 or poll2 or poll3 which has a value of None")
        } else if let (
            MyOutput::Result(value1),
            MyOutput::Result(value2),
            MyOutput::Result(value3),
        ) = (poll1_value, poll2_value, poll3_value)
        {
            assert_eq!(*value1, 20);
            assert_eq!(*value2, 30);
            assert_eq!(*value3, 100);
        }

        MyOutput::Result(100)
    }));
    poll4.after(&mut poll1).unwrap();
    poll4.after(&mut poll2).unwrap();
    poll4.after(&mut poll3).unwrap();

    cahotic.schedule_exec(poll1);
    cahotic.schedule_exec(poll3);
    cahotic.schedule_exec(poll2);
    cahotic.schedule_exec(poll4);
    cahotic.submit_packet();

    // testing 5
    let mut poll1 = Schedule::create_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        MyOutput::Result(20)
    }));
    let mut poll2 = Schedule::create_task(MyTask::Task(|| {
        sleep(Duration::from_millis(250));
        MyOutput::Result(30)
    }));

    let mut poll3 = Schedule::create_schedule(MyTask::Schedule(|schedule_vec| {
        let poll1_value = schedule_vec
            .get(0)
            .expect("poll2 accessed an invalid index on schedule_vec");

        let poll2_value = schedule_vec
            .get(1)
            .expect("poll2 accessed an invalid index on schedule_vec");

        if let (MyOutput::None, MyOutput::None) = (poll1_value, poll2_value) {
            panic!("Error, poll2 accesses poll1 or poll2 which has a value of None")
        } else if let (MyOutput::Result(value1), MyOutput::Result(value2)) =
            (poll1_value, poll2_value)
        {
            assert_eq!(*value1, 20);
            assert_eq!(*value2, 30);
        }

        MyOutput::Result(100)
    }));
    poll3.after(&mut poll1).unwrap();
    poll3.after(&mut poll2).unwrap();

    let mut poll4 = Schedule::create_schedule(MyTask::Schedule(|schedule_vec| {
        let poll1_value = schedule_vec
            .get(0)
            .expect("poll2 accessed an invalid index on schedule_vec");

        let poll2_value = schedule_vec
            .get(1)
            .expect("poll2 accessed an invalid index on schedule_vec");

        let poll3_value = schedule_vec
            .get(2)
            .expect("poll2 accessed an invalid index on schedule_vec");

        if let (MyOutput::None, MyOutput::None, MyOutput::None) =
            (poll1_value, poll2_value, poll3_value)
        {
            panic!("Error, poll2 accesses poll1 or poll2 or poll3 which has a value of None")
        } else if let (
            MyOutput::Result(value1),
            MyOutput::Result(value2),
            MyOutput::Result(value3),
        ) = (poll1_value, poll2_value, poll3_value)
        {
            assert_eq!(*value1, 20);
            assert_eq!(*value2, 30);
            assert_eq!(*value3, 100);
        }

        MyOutput::Result(100)
    }));
    poll4.after(&mut poll1).unwrap();
    poll4.after(&mut poll2).unwrap();
    poll4.after(&mut poll3).unwrap();

    cahotic.schedule_exec(poll1);
    cahotic.submit_packet();
    cahotic.schedule_exec(poll3);
    cahotic.submit_packet();
    cahotic.schedule_exec(poll2);
    cahotic.submit_packet();
    cahotic.schedule_exec(poll4);
    cahotic.submit_packet();

    cahotic.join();
}
