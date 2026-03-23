use std::sync::{Arc, atomic::AtomicU64};

use crate::{OutputTrait, PacketCore, SchedulerTrait, TaskTrait};

pub struct ListCore<F, FS, O, const PN: usize>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // id
    pub(crate) id_counter: AtomicU64,

    // handler
    pub(crate) in_task: Arc<AtomicU64>,

    // packet
    pub packet_core: PacketCore<F, FS, O, PN>,
}

impl<F, FD, O, const PN: usize> ListCore<F, FD, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> ListCore<F, FD, O, PN> {
        Self {
            // id
            id_counter: AtomicU64::new(1),

            // handler
            in_task: Arc::new(AtomicU64::new(0)),

            // packet
            packet_core: PacketCore::init(),
        }
    }

    pub fn submit_packet(&self) {
        let _ = self.packet_core.submit_packet(&self.in_task);
    }
}
