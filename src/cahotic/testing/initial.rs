use crate::{Cahotic, DependenciesVec, JobTrait, OutputTrait, TaskTrait};

enum MyOutput {
    _Result(i32),
    None,
}
impl OutputTrait for MyOutput {}

enum MyTask {
    Task(fn() -> MyOutput),
    _Schedule(fn(scheduler_vec: DependenciesVec<MyOutput>) -> MyOutput),
}

impl TaskTrait<MyOutput> for MyTask {
    fn execute(&self) -> MyOutput {
        match self {
            MyTask::Task(f) => f(),
            MyTask::_Schedule(_) => MyOutput::None,
        }
    }
}

impl JobTrait<MyOutput> for MyTask {
    fn execute(&self, scheduler_vec: DependenciesVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::_Schedule(f) => f(scheduler_vec),
        }
    }
}

#[test]
fn initial() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 4, 4096>::init().unwrap();

    // spawn task
    cahotic.spawn_task(MyTask::Task(|| MyOutput::None));

    cahotic.join();
}
