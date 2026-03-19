use std::{
    fmt::Debug,
    sync::atomic::{AtomicPtr, AtomicUsize},
};

use crate::{OutputTrait, PollWaiting, TaskDependencies, TaskTrait, TaskWithDependenciesTrait};

pub enum ExecTask<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F, &'static AtomicUsize),
    TaskWithDependencies(FS, &'static AtomicUsize),
    // DROP
    DropPoll(PollWaiting<O>, &'static AtomicUsize),
    DropDependencies(TaskDependencies<F, FS, O>, &'static AtomicUsize),

    // DROPAFTER
    DropPollAfter(
        PollWaiting<O>,
        (&'static AtomicPtr<O>, &'static AtomicUsize),
    ),
    DropDependenciesAfter(
        TaskDependencies<F, FS, O>,
        (&'static AtomicPtr<O>, &'static AtomicUsize),
    ),
    // DROPAFTER
    // DROP
    // SCHEDING
    Scheduling(
        FS,
        Vec<&'static AtomicPtr<O>>,
        AtomicUsize,
        &'static AtomicUsize,
    ),
    Output(O),
    None,
}
