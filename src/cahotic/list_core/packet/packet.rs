use std::{
    array,
    sync::atomic::{AtomicBool, AtomicUsize},
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
    pub(crate) id: usize,
    pub(crate) epoch: u64,
    //
    pub(crate) packet: [Option<WaitingTask<F, FS, O>>; PN],
    pub(crate) tail: AtomicUsize,
    pub(crate) head: AtomicUsize,
    pub(crate) done_counter: &'static AtomicUsize,
}

impl<F, FS, O, const PN: usize> Packet<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init(id: usize) -> Packet<F, FS, O, PN> {
        let packet = array::from_fn(|_| None);

        Self {
            id,
            epoch: 0,
            packet,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            done_counter: Box::leak(Box::new(AtomicUsize::new(0))),
        }
    }
}
