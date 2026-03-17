use std::{
    fmt::Debug,
    sync::atomic::{AtomicPtr, AtomicUsize},
};

use crate::{OutputTrait, PollWaiting, TaskDependencies, TaskTrait, TaskWithDependenciesTrait};

pub enum ExecTask<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F, &'static AtomicUsize),
    TaskWithDependencies(FD, &'static AtomicUsize),
    // DROP
    DropPoll(PollWaiting<O>, &'static AtomicUsize),
    DropDependencies(TaskDependencies<F, FD, O>, &'static AtomicUsize),

    // DROPAFTER
    DropPollAfter(
        PollWaiting<O>,
        (&'static AtomicPtr<O>, &'static AtomicUsize),
    ),
    DropDependenciesAfter(
        TaskDependencies<F, FD, O>,
        (&'static AtomicPtr<O>, &'static AtomicUsize),
    ),
    // DROPAFTER
    // DROP
    Output(O),
    None,
}
