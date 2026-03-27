use std::sync::atomic::Ordering;

use crate::{ExecTask, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait, ThreadUnit};

impl<F, FD, O, const PN: usize> ThreadUnit<F, FD, O, PN>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn get_idx_sch(&mut self) {
        let mut bitmap = self
            .list_core
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
            let mut bitmap = self
                .list_core
                .packet_core
                .poll_schedule_bitmap
                .fetch_and(!masking, Ordering::Release);
            bitmap &= masking;
            if bitmap != 0 {
                unsafe {
                    let schedule_slot = (*self
                        .list_core
                        .packet_core
                        .schedule_list
                        .load(Ordering::Acquire))[sch_idx]
                        .schedule
                        .take();

                    self.list_core
                        .packet_core
                        .allo_schedule_bitmap
                        .fetch_or(masking, Ordering::Release);

                    if let Some(schedule) = schedule_slot {
                        match schedule.task {
                            ExecTask::Scheduling(
                                f,
                                scheduler_vec,
                                packet_idx,
                                candidate_packet_idx,
                            ) => {
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
                                        self.list_core
                                            .packet_core
                                            .poll_schedule_bitmap
                                            .fetch_or(masking, Ordering::Release);
                                    }
                                }

                                let packet = &mut self.list_core.load_packet_list()[packet_idx];
                                let done_counter =
                                    packet.done_counter.fetch_sub(1, Ordering::Release);
                                if done_counter == 1 {
                                    self.list_core
                                        .packet_core
                                        .drop_bitmap
                                        .fetch_or(1_u64 << packet_idx, Ordering::Release);
                                }

                                // clean schedule
                                for idx in candidate_packet_idx {
                                    let idx = idx.load(Ordering::Acquire);
                                    let packet = &mut self.list_core.load_packet_list()[idx];
                                    let done_counter =
                                        packet.done_counter.fetch_sub(1, Ordering::Release);
                                    if done_counter == 1 {
                                        self.list_core
                                            .packet_core
                                            .drop_bitmap
                                            .fetch_or(1_u64 << idx, Ordering::Release);
                                    }
                                }

                                self.done_task.fetch_add(1, Ordering::Release);
                            }
                            _ => panic!(),
                        }
                    } else {
                        self.list_core
                            .packet_core
                            .poll_schedule_bitmap
                            .fetch_or(masking, Ordering::Release);
                    }
                }
            }
        }
    }
}
