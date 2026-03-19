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
    pub fn drop_pool(&self, poll_waiting: PollWaiting<O>) {
        self.in_task.fetch_add(1, Ordering::Release);
        // create return_ptr

        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task: ExecTask::DropPoll(poll_waiting, self.drop_arena.get_current_done_counter_ptr()),
            next: AtomicPtr::new(null_mut()),
            return_ptr: None,
        };

        let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

        self.drop_arena.drop(waiting_task_ptr);
    }
}
