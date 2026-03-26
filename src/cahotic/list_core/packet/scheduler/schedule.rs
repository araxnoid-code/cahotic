use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicU64, AtomicUsize, Ordering},
};

use crate::{
    ExecTask, OutputTrait, PacketCore, PollWaiting, SchedulerTrait, TaskCore, TaskTrait,
    WaitingTask,
};

pub(crate) enum ScheduleTask<F, FS, O>
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

pub struct Schedule<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // task
    pub(crate) task: ScheduleTask<F, FS, O>,

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
    pub fn create_task_schedule(&self, task: F) -> Schedule<F, FS, O> {
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        Schedule {
            task: ScheduleTask::Task(task),
            candidate_done_counter: 1,
            candidate_packet_idx: Box::leak(Box::new(AtomicUsize::new(64))),
            poll_child: vec![],
            poll_counter: None,
            candidate_packet_vec: None,
            return_ptr,
            shcedule_vec: None,
        }
    }

    pub fn create_schedule(&self, schedule: FS) -> Schedule<F, FS, O> {
        let allocated_idx = self.allocate_schedule_bitmap();

        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        let poll_counter = Box::leak(Box::new(AtomicUsize::new(0)));
        Schedule {
            task: ScheduleTask::Schedule(schedule, allocated_idx),
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
        &self,
        schedule: &mut Schedule<F, FS, O>,
        after: &mut Schedule<F, FS, O>,
    ) -> Result<(), &'static str> {
        let (allocated_idx, poll_counter) =
            if let (ScheduleTask::Schedule(_, allocated_idx), Some(poll_counter)) =
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

    pub fn add_schedule(
        &self,
        schedule: Schedule<F, FS, O>,
        id_counter: u64,
        in_task: &AtomicU64,
    ) -> PollWaiting<O> {
        unsafe {
            let mut use_packet_idx = self.use_packet.load(Ordering::Acquire) as usize;
            if use_packet_idx == 64 {
                self.set_use_packet();
                use_packet_idx = self.use_packet.load(Ordering::Acquire) as usize;
            }

            let packet = &mut (*self.packet_list.load(Ordering::Acquire))[use_packet_idx];
            let idx = packet.head.fetch_add(1, Ordering::Release);
            if idx + 1 < PN {
                // update in_task handler
                in_task.fetch_add(1, Ordering::Release);
                self.fetch_add_current_done_counter(
                    schedule.candidate_done_counter,
                    Ordering::Release,
                );
                schedule
                    .candidate_packet_idx
                    .store(use_packet_idx, Ordering::Release);

                // create return_ptr
                let return_ptr: &'static AtomicPtr<O> = schedule.return_ptr;

                if let ScheduleTask::Task(task) = schedule.task {
                    let waiting_task = WaitingTask {
                        _id: id_counter,
                        task: ExecTask::<F, FS, O>::Task(task),
                        return_ptr: Some(return_ptr),
                        poll_child: schedule.poll_child,
                    };

                    packet.task[idx] = Some(waiting_task);
                } else if let (
                    ScheduleTask::Schedule(task, allocated_idx),
                    Some(schedule_vec),
                    Some(candidate_packet),
                ) = (
                    schedule.task,
                    schedule.shcedule_vec,
                    schedule.candidate_packet_vec,
                ) {
                    let len = schedule_vec.len();
                    let waiting_task = WaitingTask {
                        _id: id_counter,
                        task: ExecTask::<F, FS, O>::Scheduling(
                            task,
                            schedule_vec,
                            if len == 0 { 0 } else { len - 1 },
                            use_packet_idx,
                            candidate_packet,
                        ),
                        return_ptr: Some(return_ptr),
                        poll_child: schedule.poll_child,
                    };
                    packet.head.fetch_sub(1, Ordering::Release);

                    *(&mut (*self.schedule_list.load(Ordering::Acquire))[allocated_idx as usize]
                        .schedule) = Some(waiting_task);
                } else {
                    panic!()
                };

                packet.drop[idx] = Some((return_ptr, Some(schedule.candidate_packet_idx)));

                PollWaiting {
                    data_ptr: return_ptr,
                }
            } else {
                packet.head.fetch_sub(1, Ordering::Release);
                let _ = self.submit_packet(in_task);
                self.add_schedule(schedule, id_counter, in_task)
            }
        }
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
        let bitmap = self.allo_schedule_bitmap.load(Ordering::Acquire);
        let index = bitmap.trailing_zeros();

        let masking = !(1_u64 << index);
        self.allo_schedule_bitmap
            .fetch_and(masking, Ordering::Release);

        index
    }
}
