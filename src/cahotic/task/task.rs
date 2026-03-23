use std::sync::atomic::AtomicPtr;

use crate::{OutputTrait, PollWaiting, SchedulerTrait, TaskTrait};

pub enum ExecTask<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F),
    DropPoll(PollWaiting<O>),
    Scheduling(FS, Vec<&'static AtomicPtr<O>>, usize, usize),
    Output(O),
    None,
}
