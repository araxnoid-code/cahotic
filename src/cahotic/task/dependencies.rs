pub struct TaskDependenciesCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) status: bool,
    pub(crate) done: AtomicBool,
    pub(crate) counter: AtomicUsize,
    pub(crate) len: AtomicU64,
    pub(crate) start: AtomicPtr<WaitingTask<F, FD, O>>, // default null, will capture the task need this task output
    pub(crate) end: AtomicPtr<WaitingTask<F, FD, O>>, // default null, will capture the task need this task output
}

use std::{
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize},
};

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
            len: AtomicU64::new(0),
            start: AtomicPtr::new(null_mut()),
            end: AtomicPtr::new(null_mut()),
        }
    }

    pub fn blank() -> TaskDependenciesCore<F, FD, O> {
        Self {
            status: false,
            done: AtomicBool::new(false),
            counter: AtomicUsize::new(0),
            len: AtomicU64::new(0),
            start: AtomicPtr::new(null_mut()),
            end: AtomicPtr::new(null_mut()),
        }
    }
}

pub struct TaskDependencies<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) task_dependencies_ptr: &'static TaskDependenciesCore<F, FD, O>,
    pub waiting_list: &'static Vec<PoolWait<O>>,
}

impl<F, FD, O> TaskDependencies<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn blank() -> TaskDependencies<F, FD, O> {
        Self {
            task_dependencies_ptr: Box::leak(Box::new(TaskDependenciesCore::blank())),
            waiting_list: Box::leak(Box::new(vec![])),
        }
    }
}

use crate::{OutputTrait, PoolWait, TaskTrait, WaitingTask};

pub trait TaskWithDependenciesTrait<O>
where
    O: OutputTrait + 'static + Send,
{
    fn execute(&self, dependencies: &'static Vec<PoolWait<O>>) -> O;

    fn is_with_dependencies(&self) -> bool {
        true
    }
}

pub trait ArrTaskDependenciesTrait<F, O, const NF: usize>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    fn task_list(self) -> [F; NF];
}

pub trait ArrTaskDependenciesWithDependenciesTrait<FD, O, const NF: usize>
where
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    fn task_list(self) -> [FD; NF];
}
