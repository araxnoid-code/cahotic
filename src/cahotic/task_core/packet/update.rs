use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
};

use crate::{
    ExecTask, OutputTrait, PacketCore, PollWaiting, SchedulerTrait, TaskTrait, WaitingTask,
};

impl<F, FS, O, const PN: usize> PacketCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // update
    pub(crate) fn _add_task(
        &self,
        task: F,
        id_counter: u64,
        in_task: &AtomicU64,
    ) -> PollWaiting<O> {
        unsafe {
            // update handler
            in_task.fetch_add(1, Ordering::Release);

            // get empty slot
            let (l2_slot, l1_slot) = self.get_l1();

            // packet
            let packet_index = l2_slot * 64 + l1_slot;
            let packet = &mut (*self.packets.load(Ordering::Relaxed))[packet_index];
            // // update handler
            packet.done_counter.fetch_add(1, Ordering::Relaxed);

            // waiting task
            // // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            // // create waiting task
            let waiting_task = WaitingTask {
                _id: id_counter,
                task: ExecTask::Task(task),
                return_ptr: Some(return_ptr),
                poll_child: vec![],
            };
            packet.task = Some(waiting_task);

            // masking (update bitmap)
            let l1_masking = 1_u64 << l1_slot;
            let l2_masking = 1_u64 << l2_slot;
            // // produsen
            let bitmap = self.pro_level1_bitmap[l2_slot].fetch_and(!l1_masking, Ordering::Relaxed);
            if (bitmap & !l1_masking) == 0 {
                // update level 2
                self.pro_level2_bitmap
                    .fetch_and(!l2_masking, Ordering::Relaxed);
                self.get_l2();
                // update drop-bitmap (later)
            }
            // // consumer
            let l1_bitmap = self.con_level1_bitmap[l2_slot].fetch_or(l1_masking, Ordering::Release);
            if l1_bitmap == 0 {
                let l2_bitmap = self.con_level2_bitmap.load(Ordering::Acquire);
                let new_l2_bitmap = l2_bitmap | (1_u64 << l2_masking);
                self.con_level2_bitmap.compare_exchange(
                    l2_bitmap,
                    new_l2_bitmap,
                    Ordering::Acquire,
                    Ordering::Release,
                );
            }

            PollWaiting {
                data_ptr: return_ptr,
            }
        }
    }

    pub(crate) fn get_l2(&self) -> u32 {
        // check l2 slot
        while self
            .pro_level2_bitmap
            .load(Ordering::Relaxed)
            .trailing_zeros()
            == 64
        {
            // spin loop if l2 slot full
            spin_loop();
        }

        // get index
        let bitmap = self.pro_level2_bitmap.load(Ordering::Acquire);
        let l2_slot = bitmap.trailing_zeros();

        // update
        self.l2_slot.store(l2_slot, Ordering::Relaxed);
        l2_slot
    }

    pub fn get_l1(&self) -> (usize, usize) {
        let mut l2_slot = self.l2_slot.load(Ordering::Relaxed) as usize;
        loop {
            let l1_slot = self.pro_level1_bitmap[l2_slot]
                .load(Ordering::Acquire)
                .trailing_zeros();

            if l1_slot != 64 {
                break (l2_slot, l1_slot as usize);
            }

            l2_slot = self.get_l2() as usize;
        }
    }
    // update
}
