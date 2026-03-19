use std::{
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering},
};

use crate::{Arena, OutputTrait, SchedulerTrait, TaskTrait, WaitingTask};

pub struct DropArena<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) arena: Arena<F, FD, O>,
}

impl<F, FD, O> DropArena<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> DropArena<F, FD, O> {
        Self {
            arena: Arena::ini(),
        }
    }

    pub fn drop(&self, waiting_task_ptr: *mut WaitingTask<F, FD, O>) {
        self.arena.drop(waiting_task_ptr);
    }

    pub fn add_current_done_counter_ptr(&self, val: usize) {
        unsafe {
            (*self.arena.done_counter.load(Ordering::Acquire)).fetch_add(val, Ordering::Release);
        }
    }

    pub fn get_current_done_counter_ptr(&self) -> *mut AtomicUsize {
        self.arena.done_counter.load(Ordering::Acquire)
    }

    pub fn drop_arena(
        &self,
    ) -> Option<(
        *mut WaitingTask<F, FD, O>,
        *mut WaitingTask<F, FD, O>,
        *mut AtomicUsize,
    )> {
        let end = self.arena.end.swap(null_mut(), Ordering::AcqRel);
        if !end.is_null() {
            let renew_counter = Box::into_raw(Box::new(AtomicUsize::new(0)));
            let done_counter = self
                .arena
                .done_counter
                .swap(renew_counter, Ordering::AcqRel);

            let start = self.arena.start.swap(null_mut(), Ordering::AcqRel);
            Some((start, end, done_counter))
        } else {
            None
        }
    }
}
