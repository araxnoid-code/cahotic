use std::sync::atomic::{AtomicPtr, AtomicUsize};

use crate::{ExecTask, SchedulerTrait};

// task
pub trait OutputTrait {}
pub trait TaskTrait<O>
where
    O: OutputTrait + 'static,
{
    fn execute(&self) -> O;

    fn is_with_dependencies() -> bool {
        false
    }
}

// scheduler
pub enum ScedulerExec<O>
where
    O: 'static + OutputTrait + Send,
{
    Exec(AtomicPtr<O>),
    Sceduler(AtomicPtr<O>, &'static AtomicUsize),
}

// WaitingTask
pub struct WaitingTask<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) _id: u64,
    pub(crate) task: ExecTask<F, FS, O>,
    pub(crate) return_ptr: Option<&'static AtomicPtr<O>>,
}
