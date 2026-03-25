use crate::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

enum MyOutput {
    _Result(i32),
    None,
}
impl OutputTrait for MyOutput {}

enum MyTask {
    Task(fn() -> MyOutput),
    _Schedule(fn(scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput),
}

impl TaskTrait<MyOutput> for MyTask {
    fn execute(&self) -> MyOutput {
        match self {
            MyTask::Task(f) => f(),
            MyTask::_Schedule(_) => MyOutput::None,
        }
    }
}

impl SchedulerTrait<MyOutput> for MyTask {
    fn execute(&self, scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::_Schedule(f) => f(scheduler_vec),
        }
    }
}

#[test]
fn initial() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    // spawn task
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));

    // submit packet
    cahotic.submit_packet();

    cahotic.join();
}
