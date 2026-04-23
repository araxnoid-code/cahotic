use std::{
    hint::spin_loop,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    ExecTask, OutputTrait, PacketCore, PollWaiting, Schedule, ScheduleTask, SchedulerTrait,
    TaskTrait, WaitingTask,
};

impl<F, FS, O, const MAX_RING_BUFFER: usize> PacketCore<F, FS, O, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn schedule_enqueue(&self, schedule: Schedule<F, FS, O>) -> PollWaiting<O> {
        unsafe {
            let mut quota_idx = self.use_quota.load(Ordering::Relaxed);
            let head = self.head.fetch_add(1, Ordering::Relaxed) & (MAX_RING_BUFFER - 1) as u64;
            if (head & ((MAX_RING_BUFFER >> 6) - 1) as u64) == 0 {
                quota_idx = self.get_quota_use();
            }

            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[head as usize];
            let quota = &mut (&mut (*self.quota_list.load(Ordering::Relaxed)))[quota_idx];
            while !packet.empty.load(Ordering::Acquire) {
                spin_loop();
            }
            schedule
                .candidate_packet_idx
                .store(quota_idx, Ordering::Relaxed);

            // update handler
            self.in_task.fetch_add(
                1 + schedule.candidate_done_counter as u64,
                Ordering::Release,
            );
            // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = schedule.return_ptr;
            // update quota counter
            quota.fetch_add(schedule.candidate_done_counter, Ordering::Relaxed);

            if let ScheduleTask::Initial(task) = schedule.task {
                let waiting_task = WaitingTask {
                    drop_handler: quota_idx,
                    _id: head,
                    task: ExecTask::<F, FS, O>::Task(task),
                    return_ptr: Some(return_ptr),
                    poll_child: schedule.poll_child,
                };

                packet.task = Some(waiting_task);
            } else if let (
                ScheduleTask::Schedule(task, allocated_idx),
                Some(schedule_vec),
                Some(candidate_packet),
            ) = (
                schedule.task,
                schedule.shcedule_vec,
                schedule.candidate_packet_vec,
            ) {
                let execute_directly = schedule_vec.len() == 0;

                let waiting_task = WaitingTask {
                    drop_handler: quota_idx,
                    _id: head,
                    task: ExecTask::<F, FS, O>::Scheduling(
                        task,
                        schedule_vec,
                        quota_idx,
                        candidate_packet,
                    ),
                    return_ptr: Some(return_ptr),
                    poll_child: schedule.poll_child,
                };

                let schedule_list =
                    &mut (*self.schedule_list.load(Ordering::Acquire))[allocated_idx as usize];

                schedule_list.schedule = Some(waiting_task);
                schedule_list.empty.store(false, Ordering::Relaxed);

                if execute_directly == true {
                    self.poll_schedule_bitmap
                        .fetch_or(1_u64 << allocated_idx, Ordering::Release);
                }
            }

            quota.push((
                return_ptr,
                Some(schedule.candidate_packet_idx),
                schedule.poll_counter,
            ));

            packet.empty.store(false, Ordering::Release);

            PollWaiting {
                data_ptr: return_ptr,
            }
        }
    }
}
