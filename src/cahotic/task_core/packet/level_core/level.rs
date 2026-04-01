use std::{ops::Deref, sync::atomic::AtomicU64};

// level2
#[repr(align(64))]
pub struct Level2 {
    bitmap: AtomicU64,
}

impl Level2 {
    pub fn init(bit: u64) -> Level2 {
        Self {
            bitmap: AtomicU64::new(bit),
        }
    }
}

impl Deref for Level2 {
    type Target = AtomicU64;
    fn deref(&self) -> &Self::Target {
        &self.bitmap
    }
}

// level 1
#[repr(align(64))]
pub struct Level1 {
    bitmap: AtomicU64,
}

impl Level1 {
    pub fn init(bit: u64) -> Level1 {
        Self {
            bitmap: AtomicU64::new(bit),
        }
    }
}

impl Deref for Level1 {
    type Target = AtomicU64;
    fn deref(&self) -> &Self::Target {
        &self.bitmap
    }
}
