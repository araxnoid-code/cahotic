use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{OutputTrait, PacketCore, ScheduleTask, SchedulerTrait, TaskTrait};

pub(crate) enum ScheduleWaitTask<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // task
    Task(F),
    // schedule, allocated schedule-bitmap
    Schedule(FS, u32),
    _Phantom(O),
}

pub struct ScheduleWait<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // task
    pub(crate) task: ScheduleWaitTask<F, FS, O>,

    // running
    pub(crate) return_ptr: &'static AtomicPtr<O>,
    pub(crate) shcedule_vec: Option<Vec<&'static AtomicPtr<O>>>,

    // submit
    pub(crate) candidate_done_counter: usize,
    pub(crate) candidate_packet_idx: &'static AtomicUsize,

    // polling
    pub(crate) poll_counter: Option<&'static AtomicUsize>,
    pub(crate) poll_child: Vec<(&'static AtomicUsize, u32)>, // (poll_counter, allocated_idx)
    pub(crate) candidate_packet_vec: Option<Vec<&'static AtomicUsize>>,
}

impl<F, FS, O, const PN: usize> PacketCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn create_task_schedule(&self, task: F) -> ScheduleWait<F, FS, O> {
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        ScheduleWait {
            task: ScheduleWaitTask::Task(task),
            candidate_done_counter: 1,
            candidate_packet_idx: Box::leak(Box::new(AtomicUsize::new(64))),
            poll_child: vec![],
            poll_counter: None,
            candidate_packet_vec: None,
            return_ptr,
            shcedule_vec: None,
        }
    }

    pub fn create_schedule(&self, schedule: FS) -> ScheduleWait<F, FS, O> {
        let allocated_idx = self.allocate_schedule_bitmap();

        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        let poll_counter = Box::leak(Box::new(AtomicUsize::new(0)));
        ScheduleWait {
            task: ScheduleWaitTask::Schedule(schedule, allocated_idx),
            return_ptr,
            candidate_done_counter: 1,
            candidate_packet_idx: Box::leak(Box::new(AtomicUsize::new(64))),
            shcedule_vec: Some(vec![]),
            candidate_packet_vec: Some(vec![]),
            poll_child: vec![],
            poll_counter: Some(poll_counter),
        }
    }

    pub fn schedule_after(
        schedule: &mut ScheduleWait<F, FS, O>,
        after: &mut ScheduleWait<F, FS, O>,
    ) -> Result<(), &'static str> {
        let (allocated_idx, poll_counter) =
            if let (ScheduleWaitTask::Schedule(_, allocated_idx), Some(poll_counter)) =
                (&schedule.task, schedule.poll_counter)
            {
                poll_counter.fetch_add(1, Ordering::Release);
                (allocated_idx, poll_counter)
            } else {
                return Err("error, next method can only be used for schedule types");
            };

        // update after
        after.candidate_done_counter += 1;
        after.poll_child.push((poll_counter, *allocated_idx));
        let return_ptr = after.return_ptr;
        let candidate_packet_idx = after.candidate_packet_idx;

        if let (Some(schedule_vec), Some(candidate_packet_vec)) = (
            &mut schedule.shcedule_vec,
            &mut schedule.candidate_packet_vec,
        ) {
            schedule_vec.push(return_ptr);
            candidate_packet_vec.push(candidate_packet_idx);
        } else {
            return Err("error, schedule_vec or candidate_packet_vec is not set in schedule");
        }

        Ok(())
    }

    pub fn allocate_schedule_bitmap(&self) -> u32 {
        while self
            .allo_schedule_bitmap
            .load(Ordering::Acquire)
            .trailing_zeros()
            == 64
        {
            spin_loop();
        }
        let index = self
            .allo_schedule_bitmap
            .load(Ordering::Acquire)
            .trailing_zeros();

        let masking = !(1_u64 << index);
        self.allo_schedule_bitmap
            .fetch_add(masking, Ordering::Release);

        index
    }
}
