use std::sync::atomic::Ordering;

use crate::{OutputTrait, Packet, PacketCore, SchedulerTrait, TaskTrait};

impl<F, FS, O, const PN: usize> PacketCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
}
