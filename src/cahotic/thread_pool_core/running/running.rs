use std::{
    hint::spin_loop,
    sync::atomic::{AtomicPtr, Ordering},
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
    pub fn running_update(&mut self) {
        loop {
            if self.join_flag.load(Ordering::Acquire) {
                break;
            }

            // SCHEDULING
            // self.get_idx_sch();
            // let sch_idx = self.use_sch_idx;
            // if sch_idx != 64 {
            //     // self.break_counter = 0;
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
            //                     let quota_idx = schedule.drop_handler.unwrap();
            //                     let done_counter = self.task_core.packet_core.quota_list[quota_idx]
            //                         .fetch_sub(1, Ordering::Relaxed);
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
            //                         let done_counter = self.task_core.packet_core.quota_list[idx]
            //                             .fetch_sub(1, Ordering::Relaxed);
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
            self.get_idx_drop();
            let drop_idx = self.use_drop_idx;
            if drop_idx != 64 {
                // if false {
                let masking = 1_u64 << drop_idx;
                let bitmap = self
                    .task_core
                    .packet_core
                    .drop_bitmap
                    .fetch_and(!masking, Ordering::Release);

                if (bitmap & masking) != 0 {
                    unsafe {
                        let quota = &mut (&mut (*self
                            .task_core
                            .packet_core
                            .quota_list
                            .load(Ordering::Relaxed)))[drop_idx];

                        quota.free();
                    }

                    self.done_task.fetch_add(1, Ordering::Relaxed);
                    self.task_core
                        .packet_core
                        .quota_bitmap
                        .fetch_or(1_u64 << drop_idx, Ordering::Release);
                }
            }

            // DROP

            let order_idx = self.order;
            let task = if order_idx != 4096 {
                let order = self.task_core.packet_core.check_order(order_idx);
                if let DequeueStatus::Ok(task) = order {
                    self.order = 4096;
                    task
                } else if let DequeueStatus::Waiting(_) = order {
                    continue;
                } else {
                    continue;
                }
            } else {
                let tail = self.task_core.packet_core.dequeue();
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
                            .packet_core
                            .poll_schedule_bitmap
                            .fetch_or(masking, Ordering::Release);
                    }
                }

                // drop packet
                unsafe {
                    let quota_idx = task.drop_handler.unwrap();
                    let quota = &(&(*self
                        .task_core
                        .packet_core
                        .quota_list
                        .load(Ordering::Relaxed)))[quota_idx];

                    let done_counter = quota.fetch_sub(1, Ordering::Relaxed);
                    if done_counter != 1 {
                        self.done_task.fetch_add(1, Ordering::Relaxed);
                    } else {
                        // println!(
                        //     "{} masking to drop idx {} cause the counter {} in task id {}",
                        //     self._id, quota_idx, done_counter, task._id
                        // );
                        self.task_core
                            .packet_core
                            .drop_bitmap
                            .fetch_or(1_u64 << quota_idx, Ordering::Release);
                    }
                }
            }
        }
    }
    ///////////////////////////////

    pub fn running(&mut self) {
        loop {
            if self.join_flag.load(Ordering::Acquire) {
                break;
            }

            // drop
            self.drop_packet();
            // drop

            // schedule poll
            self.schedule_poll();
            // schedule poll

            let packet_idx = self.use_packet_idx;
            if packet_idx == 64 {
                self.get_idx_packet();
                if self.break_counter < 100 {
                    spin_loop();
                } else if self.break_counter < 1000 {
                    yield_now();
                } else {
                    park_timeout(Duration::from_millis(10));
                    self.break_counter = 0;
                }
                self.break_counter += 1;
                continue;
            }
            self.break_counter = 0;

            let packet = &mut self.task_core.load_packet_list()[packet_idx];

            let tail = packet.tail.fetch_add(1, Ordering::Release);
            if tail + 1 == PN {
                let masking = !(1_u64 << packet_idx);
                self.task_core
                    .packet_core
                    .ready_bitmap
                    .fetch_and(masking, Ordering::Release);
            } else if tail + 1 > PN {
                self.use_packet_idx = 64;
                continue;
            }

            if let Some(task) = packet.task_list[tail].take() {
                match task.task {
                    ExecTask::Task(f) => {
                        let output = Box::into_raw(Box::new(f.execute()));
                        task.return_ptr.unwrap().store(output, Ordering::Release);

                        // update child
                        let poll_child = task.poll_child;
                        for (counter, schedule_idx) in poll_child {
                            let counter = counter.fetch_sub(1, Ordering::Release);
                            if counter == 1 {
                                let masking = 1_u64 << schedule_idx;
                                self.task_core
                                    .packet_core
                                    .poll_schedule_bitmap
                                    .fetch_or(masking, Ordering::Release);
                            }
                        }

                        // drop packet
                        self.done_task.fetch_add(1, Ordering::Release);
                        let done_counter = packet.done_counter.fetch_sub(1, Ordering::Release);
                        if done_counter == 1 {
                            self.task_core
                                .packet_core
                                .drop_bitmap
                                .fetch_or(1 << packet_idx, Ordering::Release);
                        }
                    }
                    // ExecTask::Scheduling(_, _, _, _) => {}
                    _ => panic!(),
                };
            }
        }
    }

    pub fn get_idx_packet(&mut self) {
        let mut bitmap = self
            .task_core
            .packet_core
            .ready_bitmap
            .load(Ordering::Acquire);
        let masking = self.masking_packet_idx;
        if masking != 64 {
            bitmap &= !(1_u64 << masking);
            bitmap &= !((1_u64 << masking) - 1_u64);
        }
        let index = bitmap.trailing_zeros();
        self.use_packet_idx = index as usize;
        self.masking_packet_idx = index as usize;
    }
}
