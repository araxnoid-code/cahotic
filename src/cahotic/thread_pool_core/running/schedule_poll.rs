use std::sync::atomic::Ordering;

use crate::{ExecTask, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait, ThreadUnit};

impl<F, FD, O> ThreadUnit<F, FD, O>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn get_idx_sch(&mut self) {
        let mut bitmap = self
            .packet_core
            .poll_schedule_bitmap
            .load(Ordering::Acquire);
        let masking = self.masking_sch_idx;
        if masking != 64 {
            bitmap &= !(1_u64 << masking);
            bitmap &= !((1_u64 << masking) - 1_u64);
        }
        let index = bitmap.trailing_zeros();
        self.use_sch_idx = index as usize;
        self.masking_sch_idx = index as usize;
    }

    pub fn schedule_poll(&mut self) {
        if (self.sch_counter & 31) != 0 {
            self.sch_counter += 1;
            return;
        }
        self.sch_counter = 0;

        self.get_idx_sch();
        let sch_idx = self.use_sch_idx;
        if sch_idx != 64 {
            self.break_counter = 0;
            let masking = 1_u64 << sch_idx;
            let bitmap = self
                .packet_core
                .poll_schedule_bitmap
                .fetch_and(!masking, Ordering::Release);

            if (bitmap & masking) != 0 {
                unsafe {
                    let schedule_slot =
                        &mut ((*self.packet_core.schedule_list.load(Ordering::Acquire))[sch_idx]);

                    if schedule_slot.empty.swap(true, Ordering::Relaxed) {
                        self.packet_core
                            .poll_schedule_bitmap
                            .fetch_or(masking, Ordering::Release);
                        return;
                    }

                    if let Some(schedule) = schedule_slot.schedule.take() {
                        self.packet_core
                            .allo_schedule_bitmap
                            .fetch_or(masking, Ordering::Release);

                        if let ExecTask::Scheduling(f, scheduler_vec, _, candidate_packet_idx) =
                            schedule.task
                        {
                            let output = Box::into_raw(Box::new(
                                f.execute(ScheduleVec { vec: scheduler_vec }),
                            ));
                            schedule
                                .return_ptr
                                .unwrap()
                                .store(output, Ordering::Release);

                            // update child
                            let poll_child = schedule.poll_child;
                            for (counter, schedule_idx) in poll_child {
                                let counter = counter.fetch_sub(1, Ordering::Release);
                                if counter == 1 {
                                    let masking = 1_u64 << schedule_idx;
                                    self.packet_core
                                        .poll_schedule_bitmap
                                        .fetch_or(masking, Ordering::Release);
                                }
                            }

                            // drop packet
                            let quota_idx = schedule.drop_handler;
                            let quota = &(&(*self.packet_core.quota_list.load(Ordering::Relaxed)))
                                [quota_idx];
                            let done_counter = quota.fetch_sub(1, Ordering::Relaxed);
                            if done_counter != 1 {
                                self.done_task.fetch_add(1, Ordering::Relaxed);
                            } else {
                                self.packet_core
                                    .drop_bitmap
                                    .fetch_or(1_u64 << quota_idx, Ordering::Release);
                            }

                            // drop schedule
                            for idx in candidate_packet_idx {
                                let idx = idx.load(Ordering::Acquire);
                                let quota =
                                    &(&(*self.packet_core.quota_list.load(Ordering::Relaxed)))[idx];
                                let done_counter = quota.fetch_sub(1, Ordering::Relaxed);
                                if done_counter != 1 {
                                    self.done_task.fetch_add(1, Ordering::Relaxed);
                                } else {
                                    self.packet_core
                                        .drop_bitmap
                                        .fetch_or(1_u64 << idx, Ordering::Release);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
