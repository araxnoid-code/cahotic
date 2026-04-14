use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

use crate::{OutputTrait, QuotaUnit};

impl<O, const MAX_RING_BUFFER: usize> QuotaUnit<O, MAX_RING_BUFFER>
where
    O: 'static + OutputTrait + Send,
{
    pub fn push(
        &mut self,
        value: (
            &'static AtomicPtr<O>,
            Option<&'static AtomicUsize>,
            Option<&'static AtomicUsize>,
        ),
    ) {
        let idx = self.head.fetch_add(1, Ordering::Relaxed);
        self.drop_list[idx] = Some(value);
    }
}
