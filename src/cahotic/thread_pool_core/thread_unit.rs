use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64},
    },
};

use crate::{ListCore, OutputTrait, SchedulerTrait, TaskTrait, WaitingTask};

pub struct ThreadUnit<F, FS, O, const PN: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // thread
    // // unique
    pub(crate) _id: usize,
    // // drop-stack
    pub(crate) scheduling_queue: VecDeque<WaitingTask<F, FS, O>>,

    // share
    // // thread_pool
    pub(crate) join_flag: Arc<AtomicBool>,
    pub(crate) done_task: Arc<AtomicU64>,

    // // list core
    pub(crate) list_core: Arc<ListCore<F, FS, O, PN>>,

    // packet
    pub(crate) packet_drop_queue: VecDeque<usize>,
    pub(crate) exec_packet_idx: usize,
    pub(crate) masking_packet_idx: usize,
}
