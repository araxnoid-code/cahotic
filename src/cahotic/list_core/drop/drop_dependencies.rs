use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    ExecTask, ListCore, OutputTrait, TaskDependencies, TaskTrait, TaskWithDependenciesTrait,
    WaitingTask,
};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn drop_dependencies(&self, dependencies: TaskDependencies<F, FD, O>) {
        // update in_task handler
        self.in_task.fetch_add(1, Ordering::Release);
        // create return_ptr
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task: ExecTask::DropDependencies(dependencies),
            next: AtomicPtr::new(null_mut()),
            return_ptr,
            dependencies_core_ptr: None,
            output_dependencies_ptr: None,
        };

        let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

        // swap start with new waiting task
        let pre_start_task = self.swap_start.swap(waiting_task_ptr, Ordering::AcqRel);
        unsafe {
            (*pre_start_task)
                .next
                .store(waiting_task_ptr, Ordering::Release);
            // }
        }
    }
}
