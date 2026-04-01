use std::sync::atomic::Ordering;

use crate::{OutputTrait, Packet, PollWaiting, SchedulerTrait, TaskCore, TaskTrait, WaitingTask};

impl<F, FS, O, const PN: usize> TaskCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn load_packet_list(&self) -> &mut [Packet<F, FS, O, PN>; 64] {
        unsafe {
            let packet = self.packet_core.packet_list.load(Ordering::Acquire);
            &mut (*packet)
        }
    }

    pub fn _spawn_task(&self, task: F) -> PollWaiting<O> {
        self.packet_core._add_task(
            task,
            self.id_counter.fetch_add(1, Ordering::Release),
            &self.in_task,
        )
    }
}
