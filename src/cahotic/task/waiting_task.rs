use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize};

use crate::{ExecTask, PollWaiting, TaskDependenciesCore, TaskWithDependenciesTrait};

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
pub struct WaitingTask<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) id: u64,
    pub(crate) task: ExecTask<F, FD, O>,
    pub(crate) next: AtomicPtr<WaitingTask<F, FD, O>>,
    pub(crate) return_ptr: Option<&'static AtomicPtr<O>>,
    // dependencies
    pub(crate) dependencies_core_ptr: Option<&'static TaskDependenciesCore<F, FD, O>>, // will be shared. to Waiting<O> and WaitingTask<F, O>
    pub(crate) output_dependencies_ptr: Option<&'static Vec<PollWaiting<O>>>,
}
