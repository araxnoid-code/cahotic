use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{OutputTrait, TaskTrait, TaskWithDependenciesTrait, WaitingTask};

pub struct Arena<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) start: AtomicPtr<WaitingTask<F, FD, O>>,
    pub(crate) end: AtomicPtr<WaitingTask<F, FD, O>>,
    pub(crate) done_counter: &'static AtomicUsize,
}

impl<F, FD, O> Arena<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn ini() -> Arena<F, FD, O> {
        Self {
            start: AtomicPtr::new(null_mut()),
            end: AtomicPtr::new(null_mut()),
            done_counter: Box::leak(Box::new(AtomicUsize::new(0))),
        }
    }

    pub fn drop(&self, waiting_task_ptr: *mut WaitingTask<F, FD, O>) {
        // swap start with new waiting task
        let pre_start_task = self.start.swap(waiting_task_ptr, Ordering::AcqRel);
        if !pre_start_task.is_null() {
            unsafe {
                (*pre_start_task)
                    .next
                    .store(waiting_task_ptr, Ordering::Release);
            }
        } else {
            self.end.store(waiting_task_ptr, Ordering::Release);
        }
    }
}
