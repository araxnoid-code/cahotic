use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};

#[repr(align(64))]
pub struct Adapt {
    pub(crate) time: Instant,
    pub(crate) counter: u128,
    pub(crate) counter_accumulation: AtomicU64,
    pub(crate) done_accumulation: AtomicU64,
    pub(crate) sync: AtomicBool,
}

impl Adapt {
    pub fn init(counter: u128) -> Adapt {
        Self {
            time: Instant::now(),
            counter,
            counter_accumulation: AtomicU64::new(0),
            done_accumulation: AtomicU64::new(0),
            sync: AtomicBool::new(true),
        }
    }

    pub fn adapt(&self, done_task: u64) -> Option<u64> {
        let sync = self.sync.swap(false, Ordering::Relaxed);
        if !sync {
            return None;
        }

        let count = self.time.elapsed().as_millis()
            - self.counter_accumulation.load(Ordering::Relaxed) as u128;

        let result = if count >= self.counter {
            self.counter_accumulation
                .fetch_add(count as u64, Ordering::Relaxed);

            let done_acc = self.done_accumulation.load(Ordering::Relaxed);

            self.done_accumulation
                .fetch_add(done_task - done_acc, Ordering::Relaxed);

            Some(done_acc)
        } else {
            None
        };

        self.sync.store(true, Ordering::Relaxed);

        result
    }
}
