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

pub struct PacketCore<F, FS, O, const MAX_RING_BUFFER: usize>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // handler
    pub(crate) in_task: Arc<AtomicU64>,

    // // schedule
    pub(crate) schedule_list: AtomicPtr<[ScheduleSlot<F, FS, O>; 64]>,
    //
    pub(crate) drop_bitmap: AtomicU64,
    pub(crate) allo_schedule_bitmap: AtomicU64,
    pub(crate) poll_schedule_bitmap: AtomicU64,
    //
    // update
    pub(crate) ring_buffer: AtomicPtr<Vec<Packet<F, FS, O>>>,
    pub(crate) head: HeadRingBuffer,
    pub(crate) tail: TailRingBuffer,
    // drop and quota
    pub(crate) quota_bitmap: AtomicU64,
    pub(crate) use_quota: AtomicUsize,
    pub(crate) quota_list: AtomicPtr<[QuotaUnit<O, MAX_RING_BUFFER>; 64]>,
}

impl<F, FS, O, const MAX_RING_BUFFER: usize> PacketCore<F, FS, O, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> PacketCore<F, FS, O, MAX_RING_BUFFER> {
        Self {
            // handler
            in_task: Arc::new(AtomicU64::new(0)),

            //
            schedule_list: AtomicPtr::new(Box::into_raw(Box::new(array::from_fn(|i| {
                ScheduleSlot::init(i)
            })))),
            //
            drop_bitmap: AtomicU64::new(0),
            allo_schedule_bitmap: AtomicU64::new(u64::MAX),
            poll_schedule_bitmap: AtomicU64::new(0),
            // update
            ring_buffer: AtomicPtr::new(Box::into_raw(Box::new(
                (0..MAX_RING_BUFFER)
                    .into_iter()
                    .map(|id| Packet::init(id))
                    .collect(),
            ))),
            head: HeadRingBuffer::default(),
            tail: TailRingBuffer::default(),
            //
            use_quota: AtomicUsize::new(64),
            quota_bitmap: AtomicU64::new(u64::MAX),
            quota_list: AtomicPtr::new(Box::into_raw(Box::new(array::from_fn(|_| {
                QuotaUnit::default()
            })))),
            // update
        }
    }
}
