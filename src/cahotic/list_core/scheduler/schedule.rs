use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{ExecTask, ListCore, OutputTrait, PollWaiting, SchedulerTrait, TaskTrait, WaitingTask};

pub(crate) enum ScheduleTask<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F),
    Schedule(FS),
    _Phantom(O),
}

pub struct Schedule<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) task: ScheduleTask<F, FS, O>,
    pub(crate) return_ptr: &'static AtomicPtr<O>,
    pub(crate) candidate_done_counter: usize,
    pub(crate) candidate_packet_idx: &'static AtomicUsize,
    pub(crate) shcedule_vec: Option<Vec<&'static AtomicPtr<O>>>,
    pub(crate) candidate_packet_vec: Option<Vec<&'static AtomicUsize>>,
}

impl<F, FS, O> Schedule<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn create_task(task: F) -> Self {
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        let schedule = Schedule {
            task: ScheduleTask::Task(task),
            candidate_done_counter: 1,
            candidate_packet_idx: Box::leak(Box::new(AtomicUsize::new(64))),
            candidate_packet_vec: None,
            return_ptr,
            shcedule_vec: None,
        };
        schedule
    }

    pub fn create_schedule(schedule: FS) -> Self {
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        let schedule = Schedule {
            task: ScheduleTask::Schedule(schedule),
            candidate_done_counter: 1,
            candidate_packet_idx: Box::leak(Box::new(AtomicUsize::new(64))),
            candidate_packet_vec: Some(vec![]),
            return_ptr,
            shcedule_vec: Some(vec![]),
        };
        schedule
    }

    pub fn after(&mut self, after: &mut Schedule<F, FS, O>) -> Result<(), &str> {
        if let ScheduleTask::Task(_) = self.task {
            return Err("error, next method can only be used for schedule types");
        }

        after.candidate_done_counter += 1;
        let return_ptr = after.return_ptr;
        let candidate_idx = after.candidate_packet_idx;

        if let (Some(schedule_vec), Some(candidate_idx_vec)) =
            (&mut self.shcedule_vec, &mut self.candidate_packet_vec)
        {
            schedule_vec.push(return_ptr);
            candidate_idx_vec.push(candidate_idx);
        } else {
            return Err("error, schedule_vec or candidate_idx_vec is not set in schedule");
        }

        Ok(())
    }
}

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

impl<F, FS, O, const PN: usize> ListCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn scheduler_exec(&self, schedule: Schedule<F, FS, O>) -> PollWaiting<O> {
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
