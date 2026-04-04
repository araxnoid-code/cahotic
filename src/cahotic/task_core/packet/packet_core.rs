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
    //
    // // packet
    pub packet_list: AtomicPtr<[Packet<F, FS, O, PN>; 64]>,
    pub packet_len: usize,
    // // schedule
    pub schedule_list: AtomicPtr<[ScheduleSlot<F, FS, O>; 64]>,
    //
    pub empty_bitmap: AtomicU64,
    pub ready_bitmap: AtomicU64,
    //
    pub drop_bitmap: AtomicU64,
    pub allo_schedule_bitmap: AtomicU64,
    pub poll_schedule_bitmap: AtomicU64,
    //
    pub use_packet: AtomicU32,
    //
    // update
    pub ring_buffer: AtomicPtr<Vec<Packet<F, FS, O, PN>>>,
    pub head: HeadRingBuffer,
    pub tail: TailRingBuffer,
    // drop
    pub quota_bitmap: AtomicU64,
    pub use_quota: AtomicUsize,
    pub quota_list: [QuotaCounter; 64],
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
            packet_list: AtomicPtr::new(Box::into_raw(Box::new(array::from_fn(|i| {
                Packet::init(i)
            })))),
            packet_len: PN,
            schedule_list: AtomicPtr::new(Box::into_raw(Box::new(array::from_fn(|i| {
                ScheduleSlot::init(i)
            })))),
            //
            empty_bitmap: AtomicU64::new(u64::MAX),
            ready_bitmap: AtomicU64::new(0),
            drop_bitmap: AtomicU64::new(0),
            allo_schedule_bitmap: AtomicU64::new(u64::MAX),
            poll_schedule_bitmap: AtomicU64::new(0),
            //
            use_packet: AtomicU32::new(64),
            // update
            ring_buffer: AtomicPtr::new(Box::into_raw(Box::new(
                (0..4096).into_iter().map(|id| Packet::init(id)).collect(),
            ))),
            head: HeadRingBuffer::default(),
            tail: TailRingBuffer::default(),
            //
            use_quota: AtomicUsize::new(64),
            quota_bitmap: AtomicU64::new(u64::MAX),
            quota_list: array::from_fn(|_| QuotaCounter::default()),
            // update
        }
    }

    pub(crate) fn set_use_packet(&self) {
        let empty_bitmap = self.empty_bitmap.load(Ordering::Acquire);
        let index = empty_bitmap.trailing_zeros();
        if index != 64 {
            unsafe {
                let packet = &(*self.packet_list.load(Ordering::Acquire))[index as usize];
                packet.head.store(0, Ordering::Release);
            }
            self.use_packet.store(index, Ordering::Release);
        } else {
            // waiting here
            while self.empty_bitmap.load(Ordering::Acquire).trailing_zeros() == 64 {
                spin_loop();
            }

            let index = self.empty_bitmap.load(Ordering::Acquire).trailing_zeros();

            self.use_packet.store(index, Ordering::Release);

            unsafe {
                let packet = &(*self.packet_list.load(Ordering::Acquire))[index as usize];
                packet.head.store(0, Ordering::Release);
            }
        }
    }

    pub fn load_current_done_counter(&self) -> &'static AtomicUsize {
        unsafe {
            let use_packet_idx = self.use_packet.load(Ordering::Acquire) as usize;
            let packet = &(*self.packet_list.load(Ordering::Acquire))[use_packet_idx];
            packet.done_counter
        }
    }

    pub fn fetch_add_current_done_counter(&self, val: usize, order: Ordering) {
        unsafe {
            let use_packet_idx = self.use_packet.load(Ordering::Acquire) as usize;
            let packet = &(*self.packet_list.load(Ordering::Acquire))[use_packet_idx];
            packet.done_counter.fetch_add(val, order);
        }
    }

    pub fn add_task(&self, task: F, id_counter: u64, in_task: &AtomicU64) -> PollWaiting<O> {
        unsafe {
            let mut use_packet_idx = self.use_packet.load(Ordering::Acquire) as usize;
            if use_packet_idx == 64 {
                self.set_use_packet();
                use_packet_idx = self.use_packet.load(Ordering::Acquire) as usize;
            }

            let packet = &mut (*self.packet_list.load(Ordering::Acquire))[use_packet_idx];
            let idx = packet.head.fetch_add(1, Ordering::Release);
            if idx + 1 < PN {
                // update handler
                in_task.fetch_add(1, Ordering::Release);
                self.fetch_add_current_done_counter(1, Ordering::Release);
                // create return_ptr
                let return_ptr: &'static AtomicPtr<O> =
                    Box::leak(Box::new(AtomicPtr::new(null_mut())));

                // create waiting task
                let waiting_task = WaitingTask {
                    drop_handler: None,
                    _id: id_counter,
                    task: ExecTask::Task(task),
                    return_ptr: Some(return_ptr),
                    poll_child: vec![],
                };
                packet.task_list[idx] = Some(waiting_task);
                packet.drop_list[idx] = Some((return_ptr, None, None));

                PollWaiting {
                    data_ptr: return_ptr,
                }
            } else {
                packet.head.fetch_sub(1, Ordering::Release);
                let _ = self.submit_packet(in_task);
                self.add_task(task, id_counter, in_task)
            }
        }
    }

    pub fn submit_packet(&self, in_task: &AtomicU64) -> Result<(), ()> {
        let use_packet_idx = self.use_packet.load(Ordering::Acquire);
        if use_packet_idx == 64 {
            return Err(());
        }

        in_task.fetch_add(1, Ordering::Release);
        unsafe {
            let packet = &(*self.packet_list.load(Ordering::Acquire))[use_packet_idx as usize];
            packet.tail.store(0, Ordering::Release);
        }
        let mask = 1_u64 << use_packet_idx;
        self.ready_bitmap.fetch_or(mask, Ordering::Release);
        self.empty_bitmap.fetch_and(!mask, Ordering::Release);
        self.use_packet.store(64, Ordering::Release);

        Ok(())
    }
}
