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
    Task(F),
    DropPoll(PollWaiting<O>),
    Scheduling(FS, Vec<&'static AtomicPtr<O>>, usize, usize),
    // DropArena(
    //     *mut WaitingTask<F, FS, O>,
    //     *mut WaitingTask<F, FS, O>,
    //     &'static AtomicUsize,
    // ),
    Output(O),
    None,
}
