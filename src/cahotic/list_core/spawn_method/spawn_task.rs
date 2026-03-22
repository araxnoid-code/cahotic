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
        // update handler
        self.in_task.fetch_add(1, Ordering::Release);
        self.packet_core
            .fetch_add_current_done_counter(1, Ordering::Release);
        // create return_ptr
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task: ExecTask::Task(task, self.packet_core.load_current_done_counter()),
            next: AtomicPtr::new(null_mut()),
            return_ptr: Some(return_ptr),
        };

        self.packet_core.add_task(waiting_task, &self.in_task);

        PollWaiting {
            data_ptr: return_ptr,
        }
    }
}
