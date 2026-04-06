use std::{
    array,
    ops::Deref,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::OutputTrait;

#[repr(align(64))]
pub struct QuotaCounter<O>
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

impl<O> Default for QuotaCounter<O>
where
    O: 'static + OutputTrait + Send,
{
    fn default() -> Self {
        Self {
            counter: AtomicUsize::new(64),
            head: AtomicUsize::new(0),
            drop_list: array::from_fn(|_| None),
        }
    }
}

impl<O> Deref for QuotaCounter<O>
where
    O: 'static + OutputTrait + Send,
{
    type Target = AtomicUsize;
    fn deref(&self) -> &Self::Target {
        &self.counter
    }
}
