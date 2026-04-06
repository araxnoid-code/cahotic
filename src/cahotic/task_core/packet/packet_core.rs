use std::{
    array,
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    u64,
};

use crate::{
    ExecTask, HeadRingBuffer, OutputTrait, Packet, PollWaiting, QuotaCounter, ScheduleSlot,
    SchedulerTrait, TailRingBuffer, TaskTrait, WaitingTask,
};

pub struct PacketCore<F, FS, O, const PN: usize>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // // schedule
    pub schedule_list: AtomicPtr<[ScheduleSlot<F, FS, O>; 64]>,
    //
    pub drop_bitmap: AtomicU64,
    pub allo_schedule_bitmap: AtomicU64,
    pub poll_schedule_bitmap: AtomicU64,
    //
    // update
    pub ring_buffer: AtomicPtr<Vec<Packet<F, FS, O, PN>>>,
    pub head: HeadRingBuffer,
    pub tail: TailRingBuffer,
    // drop
    pub quota_bitmap: AtomicU64,
    pub use_quota: AtomicUsize,
    pub quota_list: AtomicPtr<[QuotaCounter<O>; 64]>,
    // update
}

impl<F, FS, O, const PN: usize> PacketCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> PacketCore<F, FS, O, PN> {
        Self {
            schedule_list: AtomicPtr::new(Box::into_raw(Box::new(array::from_fn(|i| {
                ScheduleSlot::init(i)
            })))),
            //
            drop_bitmap: AtomicU64::new(0),
            allo_schedule_bitmap: AtomicU64::new(u64::MAX),
            poll_schedule_bitmap: AtomicU64::new(0),
            // update
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
            // update
        }
    }
}
