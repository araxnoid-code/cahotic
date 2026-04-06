use std::{
    array,
    ops::Deref,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize},
};

use crate::{OutputTrait, SchedulerTrait, TaskTrait, WaitingTask};

#[repr(align(64))]
pub struct Packet<F, FS, O, const PN: usize>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // id
    pub(crate) _id: usize,
    //
    pub(crate) task_list: [Option<WaitingTask<F, FS, O>>; PN],
    pub(crate) drop_list: [Option<(
        &'static AtomicPtr<O>,
        Option<&'static AtomicUsize>,
        Option<&'static AtomicUsize>,
    )>; PN],
    pub(crate) tail: AtomicUsize,
    pub(crate) head: AtomicUsize,
    pub(crate) done_counter: &'static AtomicUsize,
    // update
    pub(crate) empty: PacketEmptyStatus,
    pub(crate) task: Option<WaitingTask<F, FS, O>>,
    pub(crate) drop: Option<&'static AtomicPtr<O>>,
    // update
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
        let task = array::from_fn(|_| None);
        let drop = array::from_fn(|_| None);

        Self {
            _id: id,
            task_list: task,
            drop_list: drop,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            done_counter: Box::leak(Box::new(AtomicUsize::new(0))),
            empty: PacketEmptyStatus::default(),
            task: None,
            drop: None,
        }
    }
}
