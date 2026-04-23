use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{OutputTrait, QuotaUnit};

impl<O, const MAX_RING_BUFFER: usize> QuotaUnit<O, MAX_RING_BUFFER>
where
    O: 'static + OutputTrait + Send,
{
    pub fn free(&mut self) {
        let head = self.head.load(Ordering::Relaxed);
        for idx in 0..head {
            unsafe {
                if let Some((return_ptr, candidate, poll_counter)) = self.drop_list[idx].take() {
                    drop(Box::from_raw(
                        return_ptr.swap(null_mut(), Ordering::Acquire),
                    ));

                    drop(Box::from_raw(
                        return_ptr as *const AtomicPtr<O> as *mut AtomicPtr<O>,
                    ));

                    if let Some(candidate) = candidate {
                        drop(Box::from_raw(
                            candidate as *const AtomicUsize as *mut AtomicUsize,
                        ));
                    }

                    if let Some(counter) = poll_counter {
                        drop(Box::from_raw(
                            counter as *const AtomicUsize as *mut AtomicUsize,
                        ));
                    }
                }
            }
        }

        self.head.store(0, Ordering::Relaxed);
        self.counter.store(MAX_RING_BUFFER >> 6, Ordering::Relaxed);
    }
}
