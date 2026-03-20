use std::{
    array,
    hint::spin_loop,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    u64,
};

use crate::{OutputTrait, Packet, SchedulerTrait, TaskTrait, WaitingTask};

pub struct PacketCore<F, FS, O, const PN: usize>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    packet_list: AtomicPtr<[Packet<F, FS, O, PN>; 64]>,
    use_packet_handler: AtomicBool,
    use_packet: AtomicU32,
    empty_bitmap: AtomicU64,
    ready_bitmap: AtomicU64,
}

impl<F, FS, O, const PN: usize> PacketCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> PacketCore<F, FS, O, PN> {
        Self {
            packet_list: AtomicPtr::new(Box::into_raw(Box::new(array::from_fn(|_| {
                Packet::init()
            })))),
            empty_bitmap: AtomicU64::new(u64::MAX),
            ready_bitmap: AtomicU64::new(0),
            use_packet_handler: AtomicBool::new(false),
            use_packet: AtomicU32::new(0),
        }
    }

    fn set_use_packet(&self) {
        let empty_bitmap = self.empty_bitmap.load(Ordering::Acquire);
        let index = empty_bitmap.trailing_zeros();
        if index != 64 {
            self.use_packet.store(index, Ordering::Release);
            self.use_packet_handler.store(true, Ordering::Release);
        } else {
            // waiting here
            while self.empty_bitmap.load(Ordering::Acquire).trailing_zeros() != 64 {
                spin_loop();
            }

            self.use_packet.store(index, Ordering::Release);
            self.use_packet_handler.store(true, Ordering::Release);
        }
    }

    pub fn get_current_done_counter(&self) -> &'static AtomicUsize {
        unsafe {
            let use_packet_idx = self.use_packet.load(Ordering::Acquire) as usize;
            let packet = &(*self.packet_list.load(Ordering::Acquire))[use_packet_idx];
            packet.done_counter
        }
    }

    pub fn add_task(&self, waiting_task: WaitingTask<F, FS, O>) {
        unsafe {
            if !self.use_packet_handler.load(Ordering::Acquire) {
                self.set_use_packet();
            }

            let use_packet_idx = self.use_packet.load(Ordering::Acquire) as usize;
            if use_packet_idx >= PN {
                let _ = self.submit_packet();
                self.add_task(waiting_task);
            } else {
                let packet = &mut (*self.packet_list.load(Ordering::Acquire))[use_packet_idx];
                let idx = packet.head.fetch_add(1, Ordering::Release);
                packet.packet[idx] = Some(waiting_task);
            }
        }
    }

    pub fn submit_packet(&self) -> Result<(), ()> {
        if !self.use_packet_handler.swap(false, Ordering::Release) {
            return Err(());
        }

        let use_packet_idx = self.use_packet.load(Ordering::Acquire);
        let mask = 1_u64 << use_packet_idx;
        self.ready_bitmap.fetch_or(mask, Ordering::Release);
        self.empty_bitmap.fetch_and(!mask, Ordering::Release);

        Ok(())
    }
}
