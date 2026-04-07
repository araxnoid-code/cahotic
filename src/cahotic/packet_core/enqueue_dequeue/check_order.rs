use std::sync::atomic::Ordering;

use crate::{DequeueStatus, OutputTrait, PacketCore, SchedulerTrait, TaskTrait};

impl<F, FS, O> PacketCore<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn check_order(&self, order: usize) -> DequeueStatus<F, FS, O> {
        unsafe {
            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[order];

            if packet.empty.load(Ordering::Acquire) {
                return DequeueStatus::Waiting(order);
            }

            if let Some(task) = packet.task.take() {
                packet.empty.store(true, Ordering::Release);
                return DequeueStatus::Ok(task);
            }

            DequeueStatus::None
        }
    }
}
