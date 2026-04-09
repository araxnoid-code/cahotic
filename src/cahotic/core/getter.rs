use std::sync::atomic::Ordering;

use crate::{Cahotic, OutputTrait, SchedulerTrait, TaskTrait};

impl<F, FS, O, const N: usize> Cahotic<F, FS, O, N>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
    // bitmap
    pub fn get_allo_schedule_bitmap(&self, order: Ordering) -> u64 {
        self.packet_core.allo_schedule_bitmap.load(order)
    }

    pub fn get_poll_schedule_bitmap(&self, order: Ordering) -> u64 {
        self.packet_core.poll_schedule_bitmap.load(order)
    }
}
