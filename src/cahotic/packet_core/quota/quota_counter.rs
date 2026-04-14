use std::{
    array,
    ops::Deref,
    sync::atomic::{AtomicPtr, AtomicUsize},
};

use crate::OutputTrait;

#[repr(align(64))]
pub struct QuotaUnit<O, const MAX_RING_BUFFER: usize>
where
    O: 'static + OutputTrait + Send,
{
    pub(crate) counter: AtomicUsize,
    pub(crate) head: AtomicUsize,
    pub(crate) drop_list: [Option<(
        &'static AtomicPtr<O>,
        Option<&'static AtomicUsize>,
        Option<&'static AtomicUsize>,
    )>; 64],
}

impl<O, const MAX_RING_BUFFER: usize> Default for QuotaUnit<O, MAX_RING_BUFFER>
where
    O: 'static + OutputTrait + Send,
{
    fn default() -> Self {
        Self {
            counter: AtomicUsize::new(MAX_RING_BUFFER >> 6), // MAX_RING_BUFFER / 64
            head: AtomicUsize::new(0),
            drop_list: array::from_fn(|_| None),
        }
    }
}

impl<O, const MAX_RING_BUFFER: usize> Deref for QuotaUnit<O, MAX_RING_BUFFER>
where
    O: 'static + OutputTrait + Send,
{
    type Target = AtomicUsize;
    fn deref(&self) -> &Self::Target {
        &self.counter
    }
}
