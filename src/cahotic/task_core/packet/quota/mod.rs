use std::{ops::Deref, sync::atomic::AtomicUsize};

#[repr(align(64))]
pub struct QuotaCounter {
    counter: AtomicUsize,
}

impl Default for QuotaCounter {
    fn default() -> Self {
        Self {
            counter: AtomicUsize::new(64),
        }
    }
}

impl Deref for QuotaCounter {
    type Target = AtomicUsize;
    fn deref(&self) -> &Self::Target {
        &self.counter
    }
}
