use std::{
    sync::atomic::Ordering,
    thread::{park_timeout, yield_now},
    time::Duration,
};

use crate::{DequeueStatus, ExecTask, OutputTrait, SchedulerTrait, TaskTrait, ThreadUnit};

impl<F, FD, O, const MAX_RING_BUFFER: usize> ThreadUnit<F, FD, O, MAX_RING_BUFFER>
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

            // SCHEDULING
            self.schedule_poll();

            // DROP
            self.drop_packet();

            let order_idx = self.order;
            let task = if order_idx != MAX_RING_BUFFER {
                if let DequeueStatus::Ok(task) = self.packet_core.check_order(order_idx) {
                    self.order = MAX_RING_BUFFER;
                    Some(task)
                } else {
                    None
                }
            } else {
                let tail = self.packet_core.dequeue();
                if let DequeueStatus::Ok(task) = tail {
                    Some(task)
                } else if let DequeueStatus::Waiting(order) = tail {
                    self.order = order;
                    None
                } else {
                    None
                }
            };

            if let Some(task) = task {
                self.break_counter = 0;
                if let ExecTask::Task(f) = task.task {
                    let output = Box::into_raw(Box::new(f.execute()));
                    task.return_ptr.unwrap().store(output, Ordering::Release);

                    // update child
                    let poll_child = task.poll_child;
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
                    unsafe {
                        let quota_idx = task.drop_handler;
                        let quota =
                            &(&(*self.packet_core.quota_list.load(Ordering::Relaxed)))[quota_idx];

                        let done_counter = quota.fetch_sub(1, Ordering::Relaxed);
                        if done_counter != 1 {
                            self.done_task.fetch_add(1, Ordering::Relaxed);
                        } else {
                            self.packet_core
                                .drop_bitmap
                                .fetch_or(1_u64 << quota_idx, Ordering::Release);
                        }
                    }
                }
            } else {
                if self.break_counter < 500 {
                    yield_now();
                } else {
                    park_timeout(Duration::from_millis(10));
                }
                self.break_counter += 1;
            }
        }
    }
}
