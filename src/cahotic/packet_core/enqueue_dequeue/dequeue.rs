use std::sync::atomic::Ordering;

use crate::{DequeueStatus, OutputTrait, PacketCore, SchedulerTrait, TaskTrait};

impl<F, FS, O, const PN: usize> PacketCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn dequeue(&self) -> DequeueStatus<F, FS, O> {
        unsafe {
            let tail = self.tail.fetch_add(1, Ordering::Relaxed) & 4095;

            let packet = &mut (&mut (*self.ring_buffer.load(Ordering::Relaxed)))[tail as usize];
            if packet.empty.load(Ordering::Acquire) {
                return DequeueStatus::Waiting(tail as usize);
            }

            if let Some(task) = packet.task.take() {
                packet.empty.store(true, Ordering::Release);
                return DequeueStatus::Ok(task);
            }

            DequeueStatus::None
        }
    }
}
