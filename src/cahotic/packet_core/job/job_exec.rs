use std::{hint::spin_loop, sync::atomic::Ordering};

use crate::{
    DequeueStatus, ExecTask, Job, JobTrait, OutputTrait, PacketCore, TaskTrait, WaitingTask,
};

impl<F, FS, O, const MAX_RING_BUFFER: usize> PacketCore<F, FS, O, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + Send + 'static,
    FS: JobTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn check_job_order(&self, order: usize) -> DequeueStatus<F, FS, O> {
        unsafe {
            let job = &mut (&mut (*self.job_ring_buffer.load(Ordering::Relaxed)))[order];

            if job.empty.load(Ordering::Acquire) {
                return DequeueStatus::Waiting(order);
            }

            if let Some(task) = job.inner.take() {
                job.empty.store(true, Ordering::Release);
                return DequeueStatus::Ok(task);
            }

            DequeueStatus::None
        }
    }

    pub fn job_dequeue(&self) -> DequeueStatus<F, FS, O> {
        unsafe {
            let tail = self.job_tail.fetch_add(1, Ordering::Relaxed) & (MAX_RING_BUFFER - 1) as u64;

            let job = &mut (&mut (*self.job_ring_buffer.load(Ordering::Relaxed)))[tail as usize];
            if job.empty.load(Ordering::Acquire) {
                return DequeueStatus::Waiting(tail as usize);
            }

            if let Some(task) = job.inner.take() {
                job.empty.store(true, Ordering::Release);
                return DequeueStatus::Ok(task);
            }

            DequeueStatus::None
        }
    }

    pub fn job_enqueue(&self, job: Job<FS, O>) {
        unsafe {
            self.in_task.fetch_add(1, Ordering::Relaxed);
            let head = self.job_head.fetch_add(1, Ordering::Relaxed) & (MAX_RING_BUFFER - 1) as u64;

            let job_unit =
                &mut (&mut (*self.job_ring_buffer.load(Ordering::Relaxed)))[head as usize];

            while !job_unit.empty.load(Ordering::Acquire) {
                spin_loop();
            }

            let waiting_task = WaitingTask {
                _id: head,
                return_ptr: Some(job.inner.return_ptr),
                task: ExecTask::Job(job.clone_inner()),
                drop_handler: self.push_to_quota((job.inner.return_ptr, None, None)),
            };

            job_unit.inner = Some(waiting_task);
            job_unit.empty.store(false, Ordering::Release);
        }
    }
}
