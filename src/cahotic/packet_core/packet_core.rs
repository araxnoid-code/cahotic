use std::{
    array,
    sync::{
        Arc,
        atomic::{AtomicPtr, AtomicU64, AtomicUsize},
    },
    u64,
};

use crate::{
    HeadRingBuffer, OutputTrait, Packet, QuotaUnit, ScheduleSlot, SchedulerTrait, TailRingBuffer,
    TaskTrait,
};

/// PacketCore. The structure that manages tasks, from spawned tasks, registered schedules, join mechanisms, and quotas.
/// In short, before a task is executed, it will go through this structure first.
pub struct PacketCore<F, FS, O, const MAX_RING_BUFFER: usize>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    /// calculates each task to be executed.
    /// note: not only does it calculate the task roughly,
    /// this property is also influenced by the scheduling and drop mechanisms
    pub(crate) in_task: Arc<AtomicU64>,

    // // schedule
    pub(crate) schedule_list: AtomicPtr<[ScheduleSlot<F, FS, O>; 64]>,
    pub(crate) allo_schedule_bitmap: AtomicU64,
    pub(crate) poll_schedule_bitmap: AtomicU64,

    // ring buffer
    pub(crate) ring_buffer: AtomicPtr<Vec<Packet<F, FS, O>>>,
    pub(crate) head: HeadRingBuffer,
    pub(crate) tail: TailRingBuffer,

    /// functions to store QuotaUnit based on the order that will be identified based on its index in the array
    pub(crate) quota_list: AtomicPtr<[QuotaUnit<O, MAX_RING_BUFFER>; 64]>,

    /// serves as a map for the quota list, the main thread and thread pool will access the QuotaUnit location through this bitmap
    pub(crate) quota_bitmap: AtomicU64,

    /// serves to store QuotaUnits that are being used by PacketCore
    pub(crate) use_quota: AtomicUsize,

    /// serves to mark and notify the location of QuotaUnits that are ready to be cleaned,
    /// will be checked by the thread pool periodically
    pub(crate) drop_bitmap: AtomicU64,
}

impl<F, FS, O, const MAX_RING_BUFFER: usize> PacketCore<F, FS, O, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> PacketCore<F, FS, O, MAX_RING_BUFFER> {
        Self {
            in_task: Arc::new(AtomicU64::new(0)),

            schedule_list: AtomicPtr::new(Box::into_raw(Box::new(array::from_fn(|i| {
                ScheduleSlot::init(i)
            })))),

            drop_bitmap: AtomicU64::new(0),
            allo_schedule_bitmap: AtomicU64::new(u64::MAX),
            poll_schedule_bitmap: AtomicU64::new(0),

            ring_buffer: AtomicPtr::new(Box::into_raw(Box::new(
                (0..MAX_RING_BUFFER)
                    .into_iter()
                    .map(|id| Packet::init(id))
                    .collect(),
            ))),
            head: HeadRingBuffer::default(),
            tail: TailRingBuffer::default(),

            use_quota: AtomicUsize::new(64),
            quota_bitmap: AtomicU64::new(u64::MAX),
            quota_list: AtomicPtr::new(Box::into_raw(Box::new(array::from_fn(|_| {
                QuotaUnit::default()
            })))),
        }
    }
}
