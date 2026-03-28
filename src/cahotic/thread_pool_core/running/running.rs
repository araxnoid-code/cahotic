use std::{hint::spin_loop, sync::atomic::Ordering, thread::yield_now};

use crate::{ExecTask, OutputTrait, SchedulerTrait, TaskTrait, ThreadUnit};

impl<F, FD, O, const PN: usize> ThreadUnit<F, FD, O, PN>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
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
                                self.list_core
                                    .packet_core
                                    .poll_schedule_bitmap
                                    .fetch_or(masking, Ordering::Release);
                            }
                        }

                        // drop packet
                        self.done_task.fetch_add(1, Ordering::Release);
                        let done_counter = packet.done_counter.fetch_sub(1, Ordering::Release);
                        if done_counter == 1 {
                            self.list_core
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
}
