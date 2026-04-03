use std::{ops::Deref, sync::atomic::AtomicUsize};

// HEAD
#[repr(align(64))]
pub struct HeadRingBuffer {
    index: AtomicUsize,
}

impl Default for HeadRingBuffer {
    fn default() -> Self {
        Self {
            index: AtomicUsize::new(0),
        }
    }
}

impl Deref for HeadRingBuffer {
    type Target = AtomicUsize;
    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

// TAIL
#[repr(align(64))]
pub struct TailRingBuffer {
    index: AtomicUsize,
}

impl Deref for TailRingBuffer {
    type Target = AtomicUsize;
    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

impl Default for TailRingBuffer {
    fn default() -> Self {
        Self {
            index: AtomicUsize::new(0),
        }
    }
}
