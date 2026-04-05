use std::{
    array,
    f64::consts,
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
    pub(crate) drop_list: [Option<&'static AtomicPtr<O>>; 64],
}

impl<O> QuotaCounter<O>
where
    O: 'static + OutputTrait + Send,
{
    pub fn push(&mut self, value: &'static AtomicPtr<O>) {
        let idx = self.head.fetch_add(1, Ordering::Relaxed);
        self.drop_list[idx] = Some(value);
    }

    pub fn free(&mut self) {
        let head = self.head.load(Ordering::Relaxed);
        for idx in 0..head {
            unsafe {
                if let Some(return_ptr) = self.drop_list[idx].take() {
                    drop(Box::from_raw(
                        return_ptr.swap(null_mut(), Ordering::Relaxed),
                    ));

                    drop(Box::from_raw(
                        return_ptr as *const AtomicPtr<O> as *mut AtomicPtr<O>,
                    ));
                }
            }
        }

        self.head.store(0, Ordering::Relaxed);
        self.counter.store(64, Ordering::Relaxed);
    }
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
