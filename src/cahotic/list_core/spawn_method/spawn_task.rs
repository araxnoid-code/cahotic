use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{
    ExecTask, ListCore, OutputTrait, PollWaiting, TaskTrait, TaskWithDependenciesTrait, WaitingTask,
};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn spawn_task(&self, task: F) -> PollWaiting<O> {
        // update in_task handler
        self.in_task.fetch_add(1, Ordering::Release);
        self.drop_arena.add_current_done_counter_ptr(1);
        // create return_ptr
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        // dependencies
        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task: ExecTask::Task(task, self.drop_arena.get_current_done_counter_ptr()),
            next: AtomicPtr::new(null_mut()),
            return_ptr: Some(return_ptr),
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
        }

        self.drop_pool(PollWaiting {
            data_ptr: return_ptr,
            drop_after_caounter: Box::leak(Box::new(AtomicUsize::new(0))),
        });

        PollWaiting {
            data_ptr: return_ptr,
            drop_after_caounter: Box::leak(Box::new(AtomicUsize::new(0))),
        }
    }

    pub(crate) fn task_from_dependencies(
        &self,
        start_dep: *mut WaitingTask<F, FD, O>,
        end_dep: *mut WaitingTask<F, FD, O>,
    ) {
        let pre_start_task = self.swap_start.swap(start_dep, Ordering::AcqRel);
        unsafe {
            (*pre_start_task).next.store(end_dep, Ordering::Release);
        }
    }
}
