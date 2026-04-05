//
use crate::{
    Cahotic, ExecTask, OutputTrait, PacketCore, PollWaiting, Schedule, ScheduleTask,
    SchedulerTrait, TaskCore, TaskTrait, WaitingTask,
};
use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
};

pub enum DequeueStatus<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Ok(WaitingTask<F, FS, O>),
    Waiting(usize),
    None,
}

impl<F, FS, O, const PN: usize> PacketCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn get_quota_use(&self) -> usize {
        while self.quota_bitmap.load(Ordering::Acquire).trailing_zeros() == 64 {
            spin_loop();
        }

        let idx = self.quota_bitmap.load(Ordering::Acquire).trailing_zeros();
        self.quota_bitmap
            .fetch_and(!(1_u64 << idx), Ordering::Release);
        self.use_quota.store(idx as usize, Ordering::Relaxed);
        idx as usize
    }

    pub fn enqueue(&self, task: F, id_counter: u64, in_task: &AtomicU64) -> PollWaiting<O> {
        unsafe {
            let mut quota_idx = self.use_quota.load(Ordering::Relaxed);
            let head = self.head.fetch_add(1, Ordering::Release) & 4095;
            if (head & 63) == 0 {
                quota_idx = self.get_quota_use();
            }
            // println!("quota {} for task id {}", quota, id_counter);

            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[head as usize];
            let quota = &mut (&mut (*self.quota_list.load(Ordering::Relaxed)))[quota_idx];
            while !packet.empty.load(Ordering::Acquire) {
                spin_loop();
            }

            // update handler
            in_task.fetch_add(1, Ordering::Release);
            // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            // create waiting task
            let waiting_task = WaitingTask {
                _id: id_counter,
                task: ExecTask::Task(task),
                return_ptr: Some(return_ptr),
                poll_child: vec![],
                drop_handler: Some(quota_idx),
            };

            packet.task = Some(waiting_task);
            quota.push(return_ptr);
            packet.empty.store(false, Ordering::Release);

            PollWaiting {
                data_ptr: return_ptr,
            }
        }
    }

    pub fn schedule_enqueue(
        &self,
        schedule: Schedule<F, FS, O>,
        id_counter: u64,
        in_task: &AtomicU64,
    ) {
        unsafe {
            let mut quota = self.use_quota.load(Ordering::Relaxed);
            let head = self.head.fetch_add(1, Ordering::Release) & 4095;
            if (head & 63) == 0 {
                quota = self.get_quota_use();
            }

            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[head as usize];
            while !packet.empty.load(Ordering::Acquire) {
                spin_loop();
            }
            schedule
                .candidate_packet_idx
                .store(quota, Ordering::Release);

            // update handler
            in_task.fetch_add(1, Ordering::Release);
            // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            if let ScheduleTask::Initial(task) = schedule.task {
                let waiting_task = WaitingTask {
                    drop_handler: Some(quota),
                    _id: id_counter,
                    task: ExecTask::<F, FS, O>::Task(task),
                    return_ptr: Some(return_ptr),
                    poll_child: schedule.poll_child,
                };

                packet.task = Some(waiting_task);
                packet.drop = Some(return_ptr);
                packet.empty.store(false, Ordering::Release);

                PollWaiting {
                    data_ptr: return_ptr,
                };
            } else if let (
                ScheduleTask::Schedule(task, allocated_idx),
                Some(schedule_vec),
                Some(candidate_packet),
            ) = (
                schedule.task,
                schedule.shcedule_vec,
                schedule.candidate_packet_vec,
            ) {
                let waiting_task = WaitingTask {
                    drop_handler: Some(quota),
                    _id: id_counter,
                    task: ExecTask::<F, FS, O>::Scheduling(
                        task,
                        schedule_vec,
                        quota,
                        candidate_packet,
                    ),
                    return_ptr: Some(return_ptr),
                    poll_child: schedule.poll_child,
                };

                *(&mut (*self.schedule_list.load(Ordering::Acquire))[allocated_idx as usize]
                    .schedule) = Some(waiting_task);

                packet.drop = Some(return_ptr);
                packet.empty.store(false, Ordering::Release);

                PollWaiting {
                    data_ptr: return_ptr,
                };
            }
        }
    }

    pub fn check_order(&self, order: usize) -> DequeueStatus<F, FS, O> {
        unsafe {
            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[order];

            if packet.empty.load(Ordering::Acquire) {
                return DequeueStatus::Waiting(order);
            }

            if let Some(task) = packet.task.take() {
                packet.empty.store(true, Ordering::Release);
                return DequeueStatus::Ok(task);
            }

            DequeueStatus::None
        }
    }

    pub fn dequeue(&self) -> DequeueStatus<F, FS, O> {
        unsafe {
            let tail = self.tail.fetch_add(1, Ordering::Relaxed) & 4095;

            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[tail as usize];
            if packet.empty.load(Ordering::Acquire) {
                return DequeueStatus::Waiting(tail as usize);
            }

            if let Some(task) = packet.task.take() {
                packet.empty.store(true, Ordering::Release);
                return DequeueStatus::Ok(task);
            }

            DequeueStatus::None
        }
    }
}
//
//

//
//
impl<F, FD, O, const PN: usize> TaskCore<F, FD, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) fn spawn_task_update(&self, task: F) -> PollWaiting<O> {
        self.packet_core.enqueue(
            task,
            self.id_counter.fetch_add(1, Ordering::Relaxed),
            &self.in_task,
        )
    }
}
//
//
impl<F, FS, O, const N: usize, const PN: usize> Cahotic<F, FS, O, N, PN>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
    pub fn spawn_task_update(&self, task: F) -> PollWaiting<O> {
        self.task_core.spawn_task_update(task)
    }

    pub fn get_quota_bitmap(&self) -> u64 {
        self.task_core
            .packet_core
            .quota_bitmap
            .load(Ordering::Acquire)
    }

    pub fn get_head(&self) -> u64 {
        self.task_core.packet_core.head.load(Ordering::Acquire)
    }

    pub fn get_tail(&self) -> u64 {
        self.task_core.packet_core.tail.load(Ordering::Acquire)
    }
}
//
