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
    Task(F),
    TaskWithDependencies(FD),
    DropPoll(PollWaiting<O>),
    DropPollAfter(
        PollWaiting<O>,
        (&'static AtomicPtr<O>, &'static AtomicUsize),
    ),
    DropDependencies(TaskDependencies<F, FD, O>),
    DropDependenciesAfter(
        TaskDependencies<F, FD, O>,
        (&'static AtomicPtr<O>, &'static AtomicUsize),
    ),
    Output(O),
    None,
}
