use std::{
    ptr::{self, null_mut},
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
    usize,
};

use crate::{
    ExecTask, ListCore, OutputTrait, PollWaiting, ScheduleUnit, SchedulerTrait, TaskTrait,
    WaitingTask,
};

pub struct ScheduleVec<O>
where
    O: 'static + OutputTrait + Send,
{
    pub(crate) vec: Vec<&'static AtomicPtr<O>>,
}

impl<O> ScheduleVec<O>
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

pub struct Schedule<FS, O>
where
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) idx: usize,
    pub(crate) waiting_poll: Vec<&'static AtomicPtr<O>>,
    pub(crate) task: FS,
}

impl<FS, O> Schedule<FS, O>
where
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init(task: FS) -> Schedule<FS, O> {
        Schedule {
            idx: 0,
            waiting_poll: Vec::with_capacity(16),
            task,
        }
    }

    pub fn after(&mut self, poll_waiting: &PollWaiting<O>) {
        self.waiting_poll.push(poll_waiting.data_ptr);
        if self.waiting_poll.len() != 1 {
            self.idx += 1;
        }
    }
}

impl<F, FS, O, const PN: usize> ListCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn scheduler_exec(&self, schedule: ScheduleUnit<F, FS, O>) -> PollWaiting<O> {
        self.packet_core.add_schedule(
            schedule,
            self.id_counter.fetch_add(1, Ordering::Release),
            &self.in_task,
        )
    }

    pub(crate) fn scheduling_handler(
        &self,
        waiting_task: &mut WaitingTask<F, FS, O>,
    ) -> Result<(), ()> {
        if let ExecTask::Scheduling(_, waiting_poll, idx, _, _) = &mut waiting_task.task {
            if waiting_poll.len() == 0 {
                return Ok(());
            } else {
                let ptr = waiting_poll.get(*idx).unwrap().load(Ordering::Acquire);
                if ptr.is_null() {
                    return Err(());
                }

                if *idx == 0 {
                    return Ok(());
                } else {
                    (*idx) -= 1;
                    return Err(());
                }
            }
        } else {
            panic!()
        }
    }
}
