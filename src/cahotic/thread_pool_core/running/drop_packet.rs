use std::sync::atomic::Ordering;

use crate::{OutputTrait, SchedulerTrait, TaskTrait, ThreadUnit};

impl<F, FD, O> ThreadUnit<F, FD, O>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn get_idx_drop(&mut self) {
        let mut bitmap = self.packet_core.drop_bitmap.load(Ordering::Acquire);

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
            let bitmap = self
                .packet_core
                .drop_bitmap
                .fetch_and(!masking, Ordering::Release);
            if (bitmap & masking) != 0 {
                unsafe {
                    let quota = &mut (&mut (*self.packet_core.quota_list.load(Ordering::Relaxed)))
                        [drop_idx];

                    quota.free();
                }

                self.done_task.fetch_add(1, Ordering::Relaxed);
                self.packet_core
                    .quota_bitmap
                    .fetch_or(1_u64 << drop_idx, Ordering::Release);
            }
        }
    }
}
