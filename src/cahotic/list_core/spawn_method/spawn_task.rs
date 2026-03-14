use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    ExecTask, ListCore, OutputTrait, PoolOutput, PoolWait, TaskDependenciesCore, TaskTrait,
    TaskWithDependenciesTrait, WaitingTask,
};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn spawn_task(&self, task: F) -> PoolWait<F, FD, O> {
        // update in_task handler
        self.in_task.fetch_add(1, Ordering::SeqCst);
        // create return_ptr
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        // dependencies
        let dependencies_core_ptr = Box::leak(Box::new(TaskDependenciesCore::blank()));
        let output_dependencies_ptr = Box::leak(Box::new(Vec::new()));
        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task: ExecTask::Task(task),
            next: AtomicPtr::new(null_mut()),
            return_ptr,
            dependencies_core_ptr,
            output_dependencies_ptr,
        };

        let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

        // swap start with new waiting task
        let pre_start_task = self.swap_start.swap(waiting_task_ptr, Ordering::AcqRel);
        unsafe {
            (*pre_start_task)
                .next
                .store(waiting_task_ptr, Ordering::Release);
        }

        let pool_out = PoolOutput {
            data_ptr: return_ptr,
        };

        PoolWait {
            output: pool_out,
            dependencies_core_ptr,
            output_dependencies_ptr,
        }
    }

    pub(crate) fn task_from_dependencies(
        &self,
        start_dep: *mut WaitingTask<F, FD, O>,
        end_dep: *mut WaitingTask<F, FD, O>,
    ) {
        let pre_start_task = self.swap_start.swap(start_dep, Ordering::AcqRel);
        if !pre_start_task.is_null() {
            unsafe {
                (*pre_start_task).next.store(end_dep, Ordering::Release);
            }
        } else {
            self.swap_end.store(end_dep, Ordering::Release);
        }
    }
}
