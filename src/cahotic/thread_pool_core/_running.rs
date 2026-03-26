use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
    thread::yield_now,
};

use crate::{ExecTask, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait, ThreadUnit};

impl<F, FD, O, const PN: usize> ThreadUnit<F, FD, O, PN>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn running_packet(&mut self) {
        loop {
            if self.join_flag.load(Ordering::Acquire) {
                break;
            }

            // drop
            if self.drop_counter & 127 == 0 {
                self.get_idx_drop();
                self.drop_counter = 0;

                let drop_idx = self.use_drop_idx;
                if drop_idx != 64 {
                    let masking = 1_u64 << drop_idx;
                    let mut bitmap = self
                        .list_core
                        .packet_core
                        .drop_bitmap
                        .fetch_and(!masking, Ordering::Release);
                    bitmap &= masking;

                    if bitmap != 0 {
                        let packet = &mut self.list_core.load_packet_list()[drop_idx];
                        for i in 0..packet.head.load(Ordering::Acquire) {
                            if let Some((return_ptr, candidate_ptr)) = packet.drop[i].take() {
                                unsafe {
                                    drop(Box::from_raw(
                                        return_ptr.swap(null_mut(), Ordering::Release),
                                    ));
                                    drop(Box::from_raw(
                                        return_ptr as *const AtomicPtr<O> as *mut AtomicPtr<O>,
                                    ));
                                    if let Some(candidate_ptr) = candidate_ptr {
                                        drop(Box::from_raw(
                                            candidate_ptr as *const AtomicUsize as *mut AtomicUsize,
                                        ));
                                    }
                                }
                            }
                        }

                        self.done_task.fetch_add(1, Ordering::Release);
                        self.list_core
                            .packet_core
                            .empty_bitmap
                            .fetch_or(1_u64 << drop_idx, Ordering::Release);
                    }
                }
            }
            self.drop_counter += 1;
            // drop

            // if let Some(packet_idx) = self.packet_drop_queue.pop_front() {
            //     let packet = &mut self.list_core.load_packet_list()[packet_idx];
            //     if packet.done_counter.load(Ordering::Acquire) == 0 {
            //         for i in 0..packet.head.load(Ordering::Acquire) {
            //             if let Some((return_ptr, candidate_ptr)) = packet.drop[i].take() {
            //                 unsafe {
            //                     drop(Box::from_raw(
            //                         return_ptr.swap(null_mut(), Ordering::Release),
            //                     ));
            //                     drop(Box::from_raw(
            //                         return_ptr as *const AtomicPtr<O> as *mut AtomicPtr<O>,
            //                     ));
            //                     if let Some(candidate_ptr) = candidate_ptr {
            //                         drop(Box::from_raw(
            //                             candidate_ptr as *const AtomicUsize as *mut AtomicUsize,
            //                         ));
            //                     }
            //                 }
            //             }
            //         }

            //         // drop
            //         self.done_task.fetch_add(1, Ordering::Release);
            //         self.list_core
            //             .packet_core
            //             .empty_bitmap
            //             .fetch_or(1_u64 << packet_idx, Ordering::Release);
            //     } else {
            //         self.packet_drop_queue.push_back(packet_idx);
            //     }
            // }

            if let Some(mut schedule_task) = self.scheduling_queue.pop_front() {
                if let Ok(()) = self.list_core.scheduling_handler(&mut schedule_task) {
                    match schedule_task.task {
                        ExecTask::Scheduling(
                            f,
                            scheduler_vec,
                            _,
                            packet_idx,
                            candidate_packet_idx,
                        ) => {
                            let output = Box::into_raw(Box::new(
                                f.execute(ScheduleVec { vec: scheduler_vec }),
                            ));
                            schedule_task
                                .return_ptr
                                .unwrap()
                                .store(output, Ordering::Release);

                            // update clean
                            let packet = &mut self.list_core.load_packet_list()[packet_idx];
                            packet.done_counter.fetch_sub(1, Ordering::Release);
                            // clean schedule packet
                            for idx in candidate_packet_idx {
                                let packet = &mut self.list_core.load_packet_list()
                                    [idx.load(Ordering::Acquire)];
                                packet.done_counter.fetch_sub(1, Ordering::Release);
                            }
                            self.done_task.fetch_add(1, Ordering::Release);
                            spin_loop();
                        }
                        _ => panic!(),
                    };
                } else {
                    self.scheduling_queue.push_back(schedule_task);
                }
            }

            let packet_idx = self.use_packet_idx;
            if packet_idx == 64 {
                self.get_idx_packet();
                if self.break_counter < 1000 {
                    spin_loop();
                } else {
                    yield_now();
                }
                self.break_counter += 1;
                continue;
            }
            self.break_counter = 0;

            let packet = &mut self.list_core.load_packet_list()[packet_idx];

            let tail = packet.tail.fetch_add(1, Ordering::Release);
            if tail + 1 == PN {
                let masking = !(1_u64 << packet_idx);
                self.list_core
                    .packet_core
                    .ready_bitmap
                    .fetch_and(masking, Ordering::Release);

                // self.packet_drop_queue.push_back(packet_idx);
            } else if tail + 1 > PN {
                self.use_packet_idx = 64;
                spin_loop();
                continue;
            }

            if let Some(task) = packet.task[tail].take() {
                match task.task {
                    ExecTask::Task(f) => {
                        let output = Box::into_raw(Box::new(f.execute()));
                        task.return_ptr.unwrap().store(output, Ordering::Release);

                        self.done_task.fetch_add(1, Ordering::Release);
                        let done_counter = packet.done_counter.fetch_sub(1, Ordering::Release);
                        if done_counter == 1 {
                            self.list_core
                                .packet_core
                                .drop_bitmap
                                .fetch_or(1 << packet_idx, Ordering::Release);
                        }
                        spin_loop();
                    }
                    ExecTask::Scheduling(_, _, _, _, _) => {
                        self.scheduling_queue.push_back(task);
                    }
                    _ => panic!(),
                };
            } else {
                spin_loop();
                continue;
            }
        }
    }

    pub fn get_idx_packet(&mut self) {
        let mut bitmap = self
            .list_core
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

    pub fn get_idx_drop(&mut self) {
        let mut bitmap = self
            .list_core
            .packet_core
            .drop_bitmap
            .load(Ordering::Acquire);

        let masking = self.masking_drop_idx;
        if masking != 64 {
            bitmap &= !(1_u64 << masking);
            bitmap &= !((1_u64 << masking) - 1_u64);
        }
        let index = bitmap.trailing_zeros();
        self.use_drop_idx = index as usize;
        self.masking_drop_idx = index as usize;
    }
}
