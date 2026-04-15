use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    ExecTask, OutputTrait, PacketCore, PollWaiting, SchedulerTrait, TaskTrait, TryEnqueueStatus,
    WaitingTask,
};

impl<F, FS, O, const MAX_RING_BUFFER: usize> PacketCore<F, FS, O, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
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
            let mut quota_idx = self.use_quota.load(Ordering::Relaxed);
            let head = self.head.fetch_add(1, Ordering::Relaxed) & (MAX_RING_BUFFER - 1) as u64;
            if (head & ((MAX_RING_BUFFER >> 6) - 1) as u64) == 0 {
                quota_idx = self.get_quota_use();
            }

            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[head as usize];
            let quota = &mut (&mut (*self.quota_list.load(Ordering::Relaxed)))[quota_idx];
            while !packet.empty.load(Ordering::Acquire) {
                spin_loop();
            }

            // update handler
            self.in_task.fetch_add(1, Ordering::Release);
            // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            // create waiting task
            let waiting_task = WaitingTask {
                _id: head,
                task: ExecTask::Task(task),
                return_ptr: Some(return_ptr),
                poll_child: vec![],
                drop_handler: quota_idx,
            };

            packet.task = Some(waiting_task);
            quota.push((return_ptr, None, None));
            packet.empty.store(false, Ordering::Release);

            PollWaiting {
                data_ptr: return_ptr,
            }
        }
    }

    pub fn try_enqueue(&self, task: F) -> Result<PollWaiting<O>, TryEnqueueStatus> {
        unsafe {
            let mut quota_idx = self.use_quota.load(Ordering::Relaxed);
            let head = self.head.fetch_add(1, Ordering::Relaxed) & (MAX_RING_BUFFER - 1) as u64;
            if (head & ((MAX_RING_BUFFER >> 6) - 1) as u64) == 0 {
                quota_idx = self.try_get_quota_use()?;
            }

            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[head as usize];
            let quota = &mut (&mut (*self.quota_list.load(Ordering::Relaxed)))[quota_idx];
            if !packet.empty.load(Ordering::Acquire) {
                return Err(TryEnqueueStatus::RingBufferFull);
            }

            // update handler
            self.in_task.fetch_add(1, Ordering::Release);
            // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            // create waiting task
            let waiting_task = WaitingTask {
                _id: head,
                task: ExecTask::Task(task),
                return_ptr: Some(return_ptr),
                poll_child: vec![],
                drop_handler: quota_idx,
            };

            packet.task = Some(waiting_task);
            quota.push((return_ptr, None, None));
            packet.empty.store(false, Ordering::Release);

            Ok(PollWaiting {
                data_ptr: return_ptr,
            })
        }
    }
}
