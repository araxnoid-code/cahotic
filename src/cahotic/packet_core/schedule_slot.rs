use std::sync::atomic::AtomicBool;

use crate::{OutputTrait, SchedulerTrait, TaskTrait, WaitingTask};

#[repr(align(64))]
pub struct ScheduleSlot<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // id
    pub(crate) _id: usize,
    // status
    pub(crate) empty: AtomicBool,
    // schedule
    pub(crate) schedule: Option<WaitingTask<F, FS, O>>,
}

impl<F, FS, O> ScheduleSlot<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init(id: usize) -> ScheduleSlot<F, FS, O> {
        Self {
            _id: id,
            empty: AtomicBool::new(true),
            schedule: None,
        }
    }
}
