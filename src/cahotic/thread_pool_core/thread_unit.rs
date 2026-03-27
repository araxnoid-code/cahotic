use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64},
    },
};

use crate::{OutputTrait, SchedulerTrait, TaskCore, TaskTrait, WaitingTask};

pub struct ThreadUnit<F, FS, O, const PN: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // thread
    // // unique
    pub(crate) _id: usize,
    pub(crate) break_counter: u64,
    // // drop-stack
    pub(crate) scheduling_queue: VecDeque<WaitingTask<F, FS, O>>,

    // share
    // // thread_pool
    pub(crate) join_flag: Arc<AtomicBool>,
    pub(crate) done_task: Arc<AtomicU64>,

    // // list core
    pub(crate) list_core: Arc<TaskCore<F, FS, O, PN>>,

    // packet
    // // packet
    pub(crate) use_packet_idx: usize,
    pub(crate) masking_packet_idx: usize,
    // // drop-packet
    pub(crate) drop_counter: usize,
    pub(crate) use_drop_idx: usize,
    pub(crate) masking_drop_idx: usize,
    // // schedule
    pub(crate) sch_counter: usize,
    pub(crate) use_sch_idx: usize,
    pub(crate) masking_sch_idx: usize,
}
