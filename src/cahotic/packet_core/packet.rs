use std::{ops::Deref, sync::atomic::AtomicBool};

use crate::{OutputTrait, SchedulerTrait, TaskTrait, WaitingTask};

#[repr(align(64))]
pub struct Packet<F, FS, O, const PN: usize>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) _id: usize,
    pub(crate) empty: PacketEmptyStatus,
    pub(crate) task: Option<WaitingTask<F, FS, O>>,
}

#[repr(align(64))]
pub struct PacketEmptyStatus {
    status: AtomicBool,
}

impl Default for PacketEmptyStatus {
    fn default() -> Self {
        Self {
            status: AtomicBool::new(true),
        }
    }
}

impl Deref for PacketEmptyStatus {
    type Target = AtomicBool;
    fn deref(&self) -> &Self::Target {
        &self.status
    }
}

impl<F, FS, O, const PN: usize> Packet<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init(id: usize) -> Packet<F, FS, O, PN> {
        Self {
            _id: id,
            empty: PacketEmptyStatus::default(),
            task: None,
        }
    }
}
