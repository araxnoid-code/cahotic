use std::{
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use crate::{Arena, OutputTrait, TaskTrait, TaskWithDependenciesTrait, WaitingTask};

pub struct DropArena<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    arena_locate: AtomicBool,
    pub(crate) arena0: Arena<F, FD, O>,
    pub(crate) arena1: Arena<F, FD, O>,
}

impl<F, FD, O> DropArena<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> DropArena<F, FD, O> {
        Self {
            arena_locate: AtomicBool::new(true),
            arena0: Arena::ini(),
            arena1: Arena::ini(),
        }
    }

    pub fn drop(&self, waiting_task_ptr: *mut WaitingTask<F, FD, O>) {
        if self.arena_locate.load(Ordering::Acquire) {
            self.arena0.drop(waiting_task_ptr);
        } else {
            self.arena1.drop(waiting_task_ptr);
        }
    }

    pub fn add_current_done_counter_ptr(&self, val: usize) {
        if self.arena_locate.load(Ordering::Acquire) {
            self.arena0.done_counter.fetch_add(val, Ordering::Release);
        } else {
            self.arena1.done_counter.fetch_add(val, Ordering::Release);
        }
    }

    pub fn get_current_done_counter_ptr(&self) -> &'static std::sync::atomic::AtomicUsize {
        if self.arena_locate.load(Ordering::Acquire) {
            self.arena0.done_counter
        } else {
            self.arena1.done_counter
        }
    }

    pub fn swap_drop_arena(
        &self,
    ) -> Option<(*mut WaitingTask<F, FD, O>, *mut WaitingTask<F, FD, O>)> {
        let locate = self.arena_locate.load(Ordering::Acquire);
        let locate_target = !locate;

        if self.arena_locate.load(Ordering::Acquire) {
            let done_counter = self.arena0.done_counter.load(Ordering::Acquire);
            if done_counter == 0 {
                self.arena_locate.store(locate_target, Ordering::Release);
                // Drop
                let end = self.arena0.end.swap(null_mut(), Ordering::AcqRel);
                if !end.is_null() {
                    let start = self.arena0.start.swap(null_mut(), Ordering::AcqRel);
                    Some((start, end))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            let done_counter = self.arena1.done_counter.load(Ordering::Acquire);
            if done_counter == 0 {
                self.arena_locate.store(locate_target, Ordering::Release);
                // Drop
                let end = self.arena1.end.swap(null_mut(), Ordering::AcqRel);
                if !end.is_null() {
                    let start = self.arena1.start.swap(null_mut(), Ordering::AcqRel);
                    Some((start, end))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}
