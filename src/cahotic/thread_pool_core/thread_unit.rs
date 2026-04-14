use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64},
};

use crate::{OutputTrait, PacketCore, SchedulerTrait, TaskTrait};

pub struct ThreadUnit<F, FS, O, const MAX_RING_BUFFER: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // thread
    // // unique
    pub(crate) _id: usize,
    pub(crate) break_counter: u64,

    // share
    // // thread_pool
    pub(crate) join_flag: Arc<AtomicBool>,
    pub(crate) done_task: Arc<AtomicU64>,

    // // list core
    pub(crate) packet_core: Arc<PacketCore<F, FS, O, MAX_RING_BUFFER>>,

    // packet
    // // drop-packet
    pub(crate) drop_counter: usize,
    pub(crate) use_drop_idx: usize,
    pub(crate) masking_drop_idx: usize,
    // // schedule
    pub(crate) sch_counter: usize,
    pub(crate) use_sch_idx: usize,
    pub(crate) masking_sch_idx: usize,

    // ring buffer
    pub(crate) order: usize,
}
