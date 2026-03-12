pub struct TaskDependenciesCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) status: bool,
    pub(crate) done: AtomicBool,
    pub(crate) counter: AtomicUsize,
    pub(crate) start: AtomicPtr<WaitingTask<F, FD, O>>, // default null, will capture the task need this task output
    pub(crate) end: AtomicPtr<WaitingTask<F, FD, O>>, // default null, will capture the task need this task output
}

impl<F, FD, O> TaskDependenciesCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init(counter: usize) -> TaskDependenciesCore<F, FD, O> {
        Self {
            status: true,
            done: AtomicBool::new(false),
            counter: AtomicUsize::new(counter),
            start: AtomicPtr::new(null_mut()),
            end: AtomicPtr::new(null_mut()),
        }
    }

    pub fn blank() -> TaskDependenciesCore<F, FD, O> {
        Self {
            status: false,
            done: AtomicBool::new(false),
            counter: AtomicUsize::new(0),
            start: AtomicPtr::new(null_mut()),
            end: AtomicPtr::new(null_mut()),
        }
    }
}

use std::{
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize},
};

use crate::{OutputTrait, PoolTask, TaskTrait, WaitingTask};

pub trait TaskWithDependenciesTrait<O>
where
    O: OutputTrait + 'static + Send,
{
    fn execute(&self, dependencies: &'static Vec<PoolTask<O>>) -> O;

    fn is_with_dependencies(&self) -> bool {
        true
    }
}
