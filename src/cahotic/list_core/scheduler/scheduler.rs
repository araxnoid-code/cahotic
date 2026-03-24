use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize},
};

use crate::{
    ExecTask, OutputTrait, Schedule, SchedulerTrait, TaskTrait, WaitingTask, cahotic::task,
};

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

pub struct ScheduleUnit<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) task: ScheduleTask<F, FS, O>,
    pub(crate) return_ptr: &'static AtomicPtr<O>,
    pub(crate) candidate_done_counter: usize,
    pub(crate) candidate_packet_idx: &'static AtomicUsize,
    pub(crate) idx: usize,
    pub(crate) shcedule_vec: Option<Vec<&'static AtomicPtr<O>>>,
    pub(crate) candidate_packet_vec: Option<Vec<&'static AtomicUsize>>,
}

impl<F, FS, O> ScheduleUnit<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn create_task(task: F) -> Self {
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        let schedule = ScheduleUnit {
            task: ScheduleTask::Task(task),
            candidate_done_counter: 1,
            candidate_packet_idx: Box::leak(Box::new(AtomicUsize::new(64))),
            idx: 0,
            candidate_packet_vec: None,
            return_ptr,
            shcedule_vec: None,
        };
        schedule
    }

    pub fn create_schedule(schedule: FS) -> Self {
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        let schedule = ScheduleUnit {
            task: ScheduleTask::Schedule(schedule),
            candidate_done_counter: 1,
            candidate_packet_idx: Box::leak(Box::new(AtomicUsize::new(64))),
            idx: 0,
            candidate_packet_vec: Some(vec![]),
            return_ptr,
            shcedule_vec: Some(vec![]),
        };
        schedule
    }

    pub fn after(&mut self, after: &mut ScheduleUnit<F, FS, O>) -> Result<(), &str> {
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
