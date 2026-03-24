use std::sync::atomic::{AtomicPtr, AtomicUsize};

use crate::{OutputTrait, PollWaiting, SchedulerTrait, TaskTrait};

pub enum ExecTask<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F),
    Scheduling(
        FS,
        Vec<&'static AtomicPtr<O>>,
        usize,
        usize,
        Vec<&'static AtomicUsize>,
    ),
    Output(O),
    None,
}
