use std::sync::atomic::AtomicPtr;

use crate::{ExecTask, PoolTask, TaskWithDependenciesTrait};

// task
pub trait OutputTrait {}
pub trait TaskTrait<O>
where
    O: OutputTrait + 'static,
{
    fn exec() -> O;

    fn is_with_dependencies() -> bool {
        false
    }
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
    pub(crate) waiting_return_ptr: &'static AtomicPtr<O>,
    // dependencies
    // pub(crate) task_dependencies_core_ptr: &'static TaskDependenciesCore<F, FD, O>, // will be shared. to Waiting<O> and WaitingTask<F, O>
    pub(crate) task_dependencies_ptr: &'static Vec<PoolTask<O>>,
}
