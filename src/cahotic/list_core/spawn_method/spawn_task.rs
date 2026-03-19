use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{ExecTask, ListCore, OutputTrait, PollWaiting, SchedulerTrait, TaskTrait, WaitingTask};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn spawn_task(&self, task: F) -> PollWaiting<O> {
        // update in_task handler
        self.in_task.fetch_add(1, Ordering::Release);
        self.swap_drop_arena.add_current_done_counter_ptr(1);
        // create return_ptr
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task: ExecTask::Task(task, self.swap_drop_arena.get_current_done_counter_ptr()),
            next: AtomicPtr::new(null_mut()),
            return_ptr: Some(return_ptr),
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
}
