use std::{
    array,
    sync::{
        Arc,
        atomic::{AtomicPtr, AtomicU64, AtomicUsize},
    },
    time::Instant,
    u64,
};

use crate::{
    Adapt, HeadRingBuffer, OutputTrait, Packet, QuotaCounter, ScheduleSlot, SchedulerTrait,
    TailRingBuffer, TaskTrait,
};

pub struct PacketCore<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // handler
    pub(crate) in_task: Arc<AtomicU64>,

    // adapt
    pub(crate) adapt: Arc<Adapt>,

    // // schedule
    pub(crate) schedule_list: AtomicPtr<[ScheduleSlot<F, FS, O>; 64]>,
    pub(crate) drop_bitmap: AtomicU64,
    pub(crate) allo_schedule_bitmap: AtomicU64,
    pub(crate) poll_schedule_bitmap: AtomicU64,

    // ring buffer
    pub(crate) ring_buffer: AtomicPtr<Vec<Packet<F, FS, O>>>,
    pub(crate) head: HeadRingBuffer,
    pub(crate) tail: TailRingBuffer,
    // drop
    pub(crate) quota_bitmap: AtomicU64,
    pub(crate) use_quota: AtomicUsize,
    pub(crate) quota_list: AtomicPtr<[QuotaCounter<O>; 64]>,
}

impl<F, FS, O> PacketCore<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> PacketCore<F, FS, O> {
        Self {
            // handler
            in_task: Arc::new(AtomicU64::new(0)),
            adapt: Arc::new(Adapt::init(100)),

            //
            schedule_list: AtomicPtr::new(Box::into_raw(Box::new(array::from_fn(|i| {
                ScheduleSlot::init(i)
            })))),
            //
            drop_bitmap: AtomicU64::new(0),
            allo_schedule_bitmap: AtomicU64::new(u64::MAX),
            poll_schedule_bitmap: AtomicU64::new(0),

            ring_buffer: AtomicPtr::new(Box::into_raw(Box::new(
                (0..4096).into_iter().map(|id| Packet::init(id)).collect(),
            ))),
            head: HeadRingBuffer::default(),
            tail: TailRingBuffer::default(),
            //
            use_quota: AtomicUsize::new(64),
            quota_bitmap: AtomicU64::new(u64::MAX),
            quota_list: AtomicPtr::new(Box::into_raw(Box::new(array::from_fn(|_| {
                QuotaCounter::default()
            })))),
        }
    }
}
