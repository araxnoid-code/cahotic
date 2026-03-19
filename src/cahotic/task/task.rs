use std::{
    fmt::Debug,
    sync::atomic::{AtomicPtr, AtomicUsize},
};

use crate::{OutputTrait, PollWaiting, SchedulerTrait, TaskTrait, WaitingTask};

pub enum ExecTask<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F, *mut AtomicUsize),
    DropPoll(PollWaiting<O>, *mut AtomicUsize),
    Scheduling(
        FS,
        Vec<&'static AtomicPtr<O>>,
        AtomicUsize,
        *mut AtomicUsize,
    ),
    DropArena(
        *mut WaitingTask<F, FS, O>,
        *mut WaitingTask<F, FS, O>,
        *mut AtomicUsize,
    ),
    Output(O),
    None,
}
