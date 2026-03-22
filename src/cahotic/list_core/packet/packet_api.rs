use std::sync::atomic::Ordering;

use crate::{ListCore, OutputTrait, Packet, SchedulerTrait, TaskTrait};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn load_packet_exec_status(&self) -> bool {
        self.packet_core.exec_packet_handler.load(Ordering::Acquire)
    }

    pub fn store_packet_exec_status(&self, val: bool, order: Ordering) {
        self.packet_core.exec_packet_handler.store(val, order);
    }

    pub fn load_packet_exec_index(&self) -> usize {
        self.packet_core.exec_packet.load(Ordering::Acquire)
    }

    pub fn store_packet_exec_index(&self, val: usize, order: Ordering) {
        self.packet_core.exec_packet.store(val, order);
    }

    pub fn load_packet_list(&self) -> &mut [Packet<F, FD, O, 8>; 64] {
        unsafe {
            let packet = self.packet_core.packet_list.load(Ordering::Acquire);
            &mut (*packet)
        }
    }
}
