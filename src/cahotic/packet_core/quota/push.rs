use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

use crate::{JobTrait, OutputTrait, PacketCore, QuotaUnit, TaskTrait};

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

impl<F, FS, O, const MAX_RING_BUFFER: usize> PacketCore<F, FS, O, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + Send + 'static,
    FS: JobTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) fn push_to_quota(
        &self,
        value: (
            &'static AtomicPtr<O>,
            Option<&'static AtomicUsize>,
            Option<&'static AtomicUsize>,
        ),
    ) -> usize {
        let counter =
            self.quota_counter.fetch_add(1, Ordering::Relaxed) as usize & (MAX_RING_BUFFER - 1);

        let quota_use = if (counter & ((MAX_RING_BUFFER >> 6) - 1)) == 0 {
            self.get_quota_use()
        } else {
            self.use_quota.load(Ordering::Relaxed)
        };

        unsafe {
            let quota_unit = &mut (&mut (*self.quota_list.load(Ordering::Relaxed)))[quota_use];
            quota_unit.push(value);
        }

        quota_use
    }

    pub fn add_used_quota(&self, val: usize) {
        unsafe {
            let quota_use = self.use_quota.load(Ordering::Relaxed);
            let quota_unit = &mut (&mut (*self.quota_list.load(Ordering::Relaxed)))[quota_use];
            quota_unit.fetch_add(val, Ordering::Relaxed);
        }
    }
}
