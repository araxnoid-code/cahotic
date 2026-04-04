use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{OutputTrait, SchedulerTrait, TaskTrait, ThreadUnit};

impl<F, FD, O, const PN: usize> ThreadUnit<F, FD, O, PN>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn get_idx_drop(&mut self) {
        let mut bitmap = self
            .task_core
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

    pub fn drop_packet(&mut self) {
        if (self.drop_counter & 31) != 0 {
            self.drop_counter += 1;
            return;
        }
        self.drop_counter = 0;

        self.get_idx_drop();
        let drop_idx = self.use_drop_idx;
        if drop_idx != 64 {
            self.break_counter = 0;
            let masking = 1_u64 << drop_idx;
            let mut bitmap = self
                .task_core
                .packet_core
                .drop_bitmap
                .fetch_and(!masking, Ordering::Release);
            bitmap &= masking;

            if bitmap != 0 {
                let packet = &mut self.task_core.load_packet_list()[drop_idx];
                for i in 0..packet.head.load(Ordering::Acquire) {
                    if let Some((return_ptr, candidate_ptr, poll_counter)) =
                        packet.drop_list[i].take()
                    {
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

                            if let Some(poll_counter) = poll_counter {
                                drop(Box::from_raw(
                                    poll_counter as *const AtomicUsize as *mut AtomicUsize,
                                ));
                            }
                        }
                    }
                }

                self.done_task.fetch_add(1, Ordering::Relaxed);
                self.task_core
                    .packet_core
                    .empty_bitmap
                    .fetch_or(1_u64 << drop_idx, Ordering::Release);
            }
        }
    }
}
