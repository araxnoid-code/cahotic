use std::sync::atomic::Ordering;

use crate::{OutputTrait, Packet, SchedulerTrait, TaskCore, TaskTrait};

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
}
