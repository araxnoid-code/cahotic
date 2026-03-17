use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
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
        self.in_task.fetch_add(1, Ordering::Release);

        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task: ExecTask::DropDependencies(
                dependencies,
                self.drop_arena.get_current_done_counter_ptr(),
            ),
            next: AtomicPtr::new(null_mut()),
            return_ptr: None,
            dependencies_core_ptr: None,
            output_dependencies_ptr: None,
        };

        let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));
        self.drop_arena.drop(waiting_task_ptr);
    }
}
