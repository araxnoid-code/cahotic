use std::{
    ptr::{self, null_mut},
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
    usize,
};

use crate::{ExecTask, ListCore, OutputTrait, PollWaiting, SchedulerTrait, TaskTrait, WaitingTask};

pub struct SchedulerVec<O>
where
    O: 'static + OutputTrait + Send,
{
    pub(crate) vec: Vec<&'static AtomicPtr<O>>,
}

impl<O> SchedulerVec<O>
where
    O: 'static + OutputTrait + Send,
{
    pub fn get(&self, idx: usize) -> Option<&O> {
        unsafe {
            if let Some(ptr) = self.vec.get(idx) {
                Some(&*ptr.load(Ordering::Acquire))
            } else {
                None
            }
        }
    }
}

pub struct Scheduler<FS, O>
where
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    idx: AtomicUsize,
    waiting_poll: Vec<&'static AtomicPtr<O>>,
    task: FS,
}

impl<FS, O> Scheduler<FS, O>
where
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init(task: FS) -> Scheduler<FS, O> {
        Scheduler {
            idx: AtomicUsize::new(0),
            waiting_poll: Vec::with_capacity(16),
            task,
        }
    }

    pub fn before(&mut self, poll_waiting: &PollWaiting<O>) {
        self.waiting_poll.push(poll_waiting.data_ptr);
        if self.waiting_poll.len() != 1 {
            self.idx.fetch_add(1, Ordering::Release);
        }
    }
}

impl<F, FS, O> ListCore<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // pub fn scheduler_exec(&self, scheduler: Scheduler<FS, O>) -> PollWaiting<O> {
    //     // update in_task handler
    //     self.in_task.fetch_add(1, Ordering::Release);
    //     self.swap_drop_arena.add_current_done_counter_ptr(1);
    //     // create return_ptr
    //     let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

    //     // create waiting task
    //     let waiting_task = WaitingTask {
    //         id: self.id_counter.fetch_add(1, Ordering::Release),
    //         task: ExecTask::Scheduling(
    //             scheduler.task,
    //             scheduler.waiting_poll,
    //             scheduler.idx,
    //             self.swap_drop_arena.get_current_done_counter_ptr(),
    //         ),
    //         next: AtomicPtr::new(null_mut()),
    //         return_ptr: Some(return_ptr),
    //     };

    //     let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

    //     // swap start with new waiting task
    //     let pre_start_task = self.swap_start.swap(waiting_task_ptr, Ordering::AcqRel);
    //     unsafe {
    //         (*pre_start_task)
    //             .next
    //             .store(waiting_task_ptr, Ordering::Release);
    //     }

    //     self.drop_pool(PollWaiting {
    //         data_ptr: return_ptr,
    //         drop_after_caounter: Box::leak(Box::new(AtomicUsize::new(0))),
    //     });

    //     PollWaiting {
    //         data_ptr: return_ptr,
    //         drop_after_caounter: Box::leak(Box::new(AtomicUsize::new(0))),
    //     }
    // }

    pub(crate) fn scheduling_handler(
        &self,
        waiting_task: *mut WaitingTask<F, FS, O>,
    ) -> Option<*mut WaitingTask<F, FS, O>> {
        unsafe {
            if let ExecTask::Scheduling(_, waiting_poll, idx, _) = &(*waiting_task).task {
                if waiting_poll.len() == 0 {
                    return Some(waiting_task);
                } else {
                    let index = idx.load(Ordering::Acquire);
                    let ptr = waiting_poll.get(index).unwrap().load(Ordering::Acquire);
                    if ptr.is_null() {
                        return None;
                    }

                    if index == 0 {
                        return Some(waiting_task);
                    } else {
                        idx.fetch_sub(1, Ordering::Release);
                        return None;
                    }
                }
            } else {
                panic!()
            }
        }
    }
}
