use std::{ops::Deref, sync::atomic::AtomicU64};

// HEAD
#[repr(align(64))]
pub struct HeadRingBuffer {
    index: AtomicU64,
}

impl Default for HeadRingBuffer {
    fn default() -> Self {
        Self {
            index: AtomicU64::new(0),
        }
    }
}

impl Deref for HeadRingBuffer {
    type Target = AtomicU64;
    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

// TAIL
#[repr(align(64))]
pub struct TailRingBuffer {
    index: AtomicU64,
}

impl Deref for TailRingBuffer {
    type Target = AtomicU64;
    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

impl Default for TailRingBuffer {
    fn default() -> Self {
        Self {
            index: AtomicU64::new(0),
        }
    }
}
