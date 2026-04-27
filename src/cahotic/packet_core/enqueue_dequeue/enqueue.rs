use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    ExecTask, JobTrait, OutputTrait, PacketCore, PollWaiting, TaskTrait, TryEnqueueStatus,
    WaitingTask,
};

impl<F, FS, O, const MAX_RING_BUFFER: usize> PacketCore<F, FS, O, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + Send + 'static,
    FS: JobTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn get_quota_use(&self) -> usize {
        while self.quota_bitmap.load(Ordering::Acquire).trailing_zeros() == 64 {
            spin_loop();
        }

        let idx = self.quota_bitmap.load(Ordering::Acquire).trailing_zeros();
        self.quota_bitmap
            .fetch_and(!(1_u64 << idx), Ordering::Relaxed);
        self.use_quota.store(idx as usize, Ordering::Relaxed);
        idx as usize
    }

    pub fn try_get_quota_use(&self) -> Result<usize, TryEnqueueStatus> {
        if self.quota_bitmap.load(Ordering::Acquire).trailing_zeros() == 64 {
            return Err(TryEnqueueStatus::QuotaFull);
        }

        let idx = self.quota_bitmap.load(Ordering::Acquire).trailing_zeros();
        self.quota_bitmap
            .fetch_and(!(1_u64 << idx), Ordering::Relaxed);
        self.use_quota.store(idx as usize, Ordering::Relaxed);
        Ok(idx as usize)
    }

    //

    pub fn enqueue(&self, task: F) -> PollWaiting<O> {
        unsafe {
            self.in_task.fetch_add(1, Ordering::Release);
            let head = self.head.fetch_add(1, Ordering::Relaxed) & (MAX_RING_BUFFER - 1) as u64;

            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[head as usize];
            while !packet.empty.load(Ordering::Acquire) {
                spin_loop();
            }

            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            let waiting_task = WaitingTask {
                _id: head,
                task: ExecTask::Task(task),
                return_ptr: Some(return_ptr),
                drop_handler: self.push_to_quota((return_ptr, None, None)),
            };

            packet.task = Some(waiting_task);
            packet.empty.store(false, Ordering::Release);

            PollWaiting {
                data_ptr: return_ptr,
            }
        }
    }

    pub fn try_enqueue(&self, task: F) -> Result<PollWaiting<O>, TryEnqueueStatus> {
        unsafe {
            self.in_task.fetch_add(1, Ordering::Release);
            let head = self.head.fetch_add(1, Ordering::Relaxed) & (MAX_RING_BUFFER - 1) as u64;

            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[head as usize];
            if !packet.empty.load(Ordering::Acquire) {
                self.in_task.fetch_sub(1, Ordering::Release);
                return Err(TryEnqueueStatus::RingBufferFull);
            }

            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            let waiting_task = WaitingTask {
                _id: head,
                task: ExecTask::Task(task),
                return_ptr: Some(return_ptr),
                drop_handler: self.push_to_quota((return_ptr, None, None)),
            };

            packet.task = Some(waiting_task);
            packet.empty.store(false, Ordering::Release);

            Ok(PollWaiting {
                data_ptr: return_ptr,
            })
        }
    }
}
