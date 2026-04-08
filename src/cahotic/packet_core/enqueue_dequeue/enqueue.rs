use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    ExecTask, OutputTrait, PacketCore, PollWaiting, SchedulerTrait, TaskTrait, WaitingTask,
};

impl<F, FS, O> PacketCore<F, FS, O>
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
            .fetch_and(!(1_u64 << idx), Ordering::Release);
        self.use_quota.store(idx as usize, Ordering::Relaxed);
        idx as usize
    }

    pub fn enqueue(&self, task: F) -> PollWaiting<O> {
        unsafe {
            let mut quota_idx = self.use_quota.load(Ordering::Relaxed);
            let head = self.head.fetch_add(1, Ordering::Release) & 4095;
            if (head & 63) == 0 {
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
}
