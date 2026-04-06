use std::sync::atomic::Ordering;

use crate::{OutputTrait, Packet, SchedulerTrait, TaskCore, TaskTrait};

impl<F, FS, O, const PN: usize> TaskCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
}
