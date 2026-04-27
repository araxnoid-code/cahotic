use std::sync::atomic::Ordering;

use crate::{DequeueStatus, ExecTask, Job, JobTrait, JobVec, OutputTrait, TaskTrait, ThreadUnit};

impl<F, FD, O, const MAX_RING_BUFFER: usize> ThreadUnit<F, FD, O, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + 'static + Send,
    FD: JobTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn job_running(&mut self) {
        let order_idx = self.job_order;
        let task = if order_idx != MAX_RING_BUFFER {
            if let DequeueStatus::Ok(task) = self.packet_core.check_job_order(order_idx) {
                self.job_order = MAX_RING_BUFFER;
                Some(task)
            } else {
                None
            }
        } else {
            let tail = self.packet_core.job_dequeue();
            if let DequeueStatus::Ok(task) = tail {
                Some(task)
            } else if let DequeueStatus::Waiting(order) = tail {
                self.job_order = order;
                None
            } else {
                None
            }
        };

        if let Some(task) = task {
            self.break_counter = 0;

            if let ExecTask::Job(job) = task.task {
                let sch_vec = JobVec {
                    vec: job.return_ptr_list.take(),
                };

                let output = Box::into_raw(Box::new(job.task.execute(sch_vec)));
                job.return_ptr.store(output, Ordering::Release);

                let quota_idx = task.drop_handler;
                for child in job.child_list.take() {
                    child.push_parent_quota(quota_idx);
                    let counter = child.exec_counter.fetch_sub(1, Ordering::Relaxed);

                    if counter == 1 {
                        let job = Job { inner: child };
                        self.packet_core.job_enqueue(job);
                    }
                }

                unsafe {
                    let counter = (*self.packet_core.quota_list.load(Ordering::Relaxed))[quota_idx]
                        .counter
                        .fetch_sub(1, Ordering::Relaxed);

                    if counter != 1 {
                        self.done_task.fetch_add(1, Ordering::Relaxed);
                    } else {
                        self.packet_core
                            .drop_bitmap
                            .fetch_or(1 << quota_idx, Ordering::Release);
                    }

                    for parent_quota_idx in job.parent_quota.borrow().iter() {
                        let idx = parent_quota_idx.load(Ordering::Relaxed);
                        let counter = (*self.packet_core.quota_list.load(Ordering::Relaxed))[idx]
                            .counter
                            .fetch_sub(1, Ordering::Relaxed);

                        if counter == 1 {
                            self.packet_core
                                .drop_bitmap
                                .fetch_or(1 << quota_idx, Ordering::Release);
                        }
                    }
                }
            }
        }
    }
}
