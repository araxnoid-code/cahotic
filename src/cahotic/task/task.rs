use std::{
    fmt::Debug,
    sync::atomic::{AtomicPtr, AtomicUsize},
};

use crate::{OutputTrait, PollWaiting, SchedulerTrait, TaskTrait};

pub enum ExecTask<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F, &'static AtomicUsize),
    DropPoll(PollWaiting<O>, &'static AtomicUsize),

    Scheduling(
        FS,
        Vec<&'static AtomicPtr<O>>,
        AtomicUsize,
        &'static AtomicUsize,
    ),
    Output(O),
    None,
}
