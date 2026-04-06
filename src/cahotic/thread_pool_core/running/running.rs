use std::{
    hint::spin_loop,
    sync::atomic::Ordering,
    thread::{park_timeout, yield_now},
    time::Duration,
};

use crate::{
    DequeueStatus, ExecTask, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait, ThreadUnit,
};

impl<F, FD, O, const PN: usize> ThreadUnit<F, FD, O, PN>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    //
    pub fn running(&mut self) {
        loop {
            if self.join_flag.load(Ordering::Acquire) {
                break;
            }

            // SCHEDULING
            self.schedule_poll();
            // self.get_idx_sch();
            // let sch_idx = self.use_sch_idx;
            // if sch_idx != 64 {
            //     self.break_counter = 0;
            //     let masking = 1_u64 << sch_idx;
            //     let bitmap = self
            //         .task_core
            //         .packet_core
            //         .poll_schedule_bitmap
            //         .fetch_and(!masking, Ordering::Release);

            //     if (bitmap & masking) != 0 {
            //         unsafe {
            //             let schedule_slot = (*self
            //                 .task_core
            //                 .packet_core
            //                 .schedule_list
            //                 .load(Ordering::Acquire))[sch_idx]
            //                 .schedule
            //                 .take();

            //             self.task_core
            //                 .packet_core
            //                 .allo_schedule_bitmap
            //                 .fetch_or(masking, Ordering::Release);

            //             if let Some(schedule) = schedule_slot {
            //                 if let ExecTask::Scheduling(f, scheduler_vec, _, candidate_packet_idx) =
            //                     schedule.task
            //                 {
            //                     let output = Box::into_raw(Box::new(
            //                         f.execute(ScheduleVec { vec: scheduler_vec }),
            //                     ));
            //                     schedule
            //                         .return_ptr
            //                         .unwrap()
            //                         .store(output, Ordering::Release);

            //                     // update child
            //                     let poll_child = schedule.poll_child;
            //                     for (counter, schedule_idx) in poll_child {
            //                         let counter = counter.fetch_sub(1, Ordering::Release);
            //                         if counter == 1 {
            //                             let masking = 1_u64 << schedule_idx;
            //                             self.task_core
            //                                 .packet_core
            //                                 .poll_schedule_bitmap
            //                                 .fetch_or(masking, Ordering::Release);
            //                         }
            //                     }

            //                     // drop packet
            //                     let quota_idx = schedule.drop_handler;
            //                     let quota = &(&(*self
            //                         .task_core
            //                         .packet_core
            //                         .quota_list
            //                         .load(Ordering::Relaxed)))[quota_idx];
            //                     let done_counter = quota.fetch_sub(1, Ordering::Relaxed);
            //                     if done_counter != 1 {
            //                         self.done_task.fetch_add(1, Ordering::Relaxed);
            //                     } else {
            //                         self.task_core
            //                             .packet_core
            //                             .drop_bitmap
            //                             .fetch_or(1_u64 << quota_idx, Ordering::Release);
            //                     }

            //                     // drop schedule
            //                     for idx in candidate_packet_idx {
            //                         let idx = idx.load(Ordering::Acquire);
            //                         let quota = &(&(*self
            //                             .task_core
            //                             .packet_core
            //                             .quota_list
            //                             .load(Ordering::Relaxed)))[idx];
            //                         let done_counter = quota.fetch_sub(1, Ordering::Relaxed);
            //                         if done_counter != 1 {
            //                             self.done_task.fetch_add(1, Ordering::Relaxed);
            //                         } else {
            //                             self.task_core
            //                                 .packet_core
            //                                 .drop_bitmap
            //                                 .fetch_or(1_u64 << idx, Ordering::Release);
            //                         }
            //                     }
            //                 }
            //             } else {
            //                 self.task_core
            //                     .packet_core
            //                     .poll_schedule_bitmap
            //                     .fetch_or(masking, Ordering::Release);
            //             }
            //         }
            //     }
            // }
            // SCHEDULING

            // DROP
            self.drop_packet();
            // self.get_idx_drop();
            // let drop_idx = self.use_drop_idx;
            // if drop_idx != 64 {
            //     let masking = 1_u64 << drop_idx;
            //     let bitmap = self
            //         .task_core
            //         .packet_core
            //         .drop_bitmap
            //         .fetch_and(!masking, Ordering::Release);
            //     if (bitmap & masking) != 0 {
            //         unsafe {
            //             let quota = &mut (&mut (*self
            //                 .task_core
            //                 .packet_core
            //                 .quota_list
            //                 .load(Ordering::Relaxed)))[drop_idx];

            //             quota.free();
            //         }

            //         self.done_task.fetch_add(1, Ordering::Relaxed);
            //         self.task_core
            //             .packet_core
            //             .quota_bitmap
            //             .fetch_or(1_u64 << drop_idx, Ordering::Release);
            //     }
            // }
            // DROP

            let order_idx = self.order;
            let task = if order_idx != 4096 {
                let order = self.task_core.check_order(order_idx);
                if let DequeueStatus::Ok(task) = order {
                    self.order = 4096;
                    task
                } else if let DequeueStatus::Waiting(_) = order {
                    continue;
                } else {
                    continue;
                }
            } else {
                let tail = self.task_core.dequeue();
                if let DequeueStatus::Ok(task) = tail {
                    task
                } else if let DequeueStatus::Waiting(order) = tail {
                    self.order = order;
                    continue;
                } else {
                    continue;
                }
            };

            if let ExecTask::Task(f) = task.task {
                let output = Box::into_raw(Box::new(f.execute()));
                task.return_ptr.unwrap().store(output, Ordering::Release);

                // update child
                let poll_child = task.poll_child;
                for (counter, schedule_idx) in poll_child {
                    let counter = counter.fetch_sub(1, Ordering::Release);
                    if counter == 1 {
                        let masking = 1_u64 << schedule_idx;
                        self.task_core
                            .poll_schedule_bitmap
                            .fetch_or(masking, Ordering::Release);
                    }
                }

                // drop packet
                unsafe {
                    let quota_idx = task.drop_handler;
                    let quota = &(&(*self.task_core.quota_list.load(Ordering::Relaxed)))[quota_idx];

                    let done_counter = quota.fetch_sub(1, Ordering::Relaxed);
                    if done_counter != 1 {
                        self.done_task.fetch_add(1, Ordering::Relaxed);
                    } else {
                        self.task_core
                            .drop_bitmap
                            .fetch_or(1_u64 << quota_idx, Ordering::Release);
                    }
                }
            }
        }
    }
    ///////////////////////////////
}
