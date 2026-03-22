use std::{hint::spin_loop, sync::atomic::Ordering};

use crate::{ExecTask, OutputTrait, SchedulerTrait, TaskTrait, ThreadUnit};

impl<F, FD, O> ThreadUnit<F, FD, O>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn running_packet(&mut self) {
        loop {
            if self.join_flag.load(Ordering::Acquire) {
                break;
            }

            if let Some(packet_idx) = self.packet_drop.pop_front() {
                let packet = &mut self.list_core.load_packet_list()[packet_idx];
                if packet.done_counter.load(Ordering::Acquire) == 0 {
                    // drop
                    // waiting
                    // drop
                    self.done_task.fetch_add(1, Ordering::Release);
                    self.list_core
                        .packet_core
                        .empty_bitmap
                        .fetch_or(1_u64 << packet_idx, Ordering::Release);
                } else {
                    self.packet_drop.push_back(packet_idx);
                }
            }

            if let Some(()) = self.restock_exec_packet() {
                let index = self.list_core.load_packet_exec_index();
                let packet = &mut self.list_core.load_packet_list()[index];
                let len = self.list_core.packet_core.packet_len;
                let tail = packet.tail.fetch_add(1, Ordering::Release);
                if tail + 1 == len {
                    self.list_core
                        .store_packet_exec_status(false, Ordering::Release);
                    self.packet_drop.push_back(index);
                } else if tail + 1 > len {
                    spin_loop();
                    continue;
                }

                if let Some(task) = packet.packet[tail].take() {
                    match task.task {
                        ExecTask::Task(f, done_arena_counter) => {
                            let output = Box::into_raw(Box::new(f.execute()));
                            task.return_ptr.unwrap().store(output, Ordering::Release);

                            (*done_arena_counter).fetch_sub(1, Ordering::Release);
                            self.done_task.fetch_add(1, Ordering::Release);
                            spin_loop();
                        }
                        _ => panic!(),
                    };
                } else {
                    spin_loop();
                    continue;
                }
            }
        }
    }

    pub fn restock_exec_packet(&self) -> Option<()> {
        if !self.list_core.load_packet_exec_status() {
            let is_reprt = self.reprt_handler.swap(false, Ordering::AcqRel);
            if is_reprt {
                if self.list_core.load_packet_exec_status() {
                    self.reprt_handler.store(true, Ordering::Release);
                    spin_loop();
                    return None;
                }

                let mut ready_bitmap = self
                    .list_core
                    .packet_core
                    .ready_bitmap
                    .load(Ordering::Acquire);

                let prev_exec_index = self.list_core.load_packet_exec_index();
                let masking_on = !(1_u64 << prev_exec_index);
                let masking_before = !((1_u64 << prev_exec_index) - 1_u64);
                ready_bitmap &= masking_before;

                let index = ready_bitmap.trailing_zeros();
                if index >= 64 {
                    // empty
                    self.list_core.store_packet_exec_index(0, Ordering::Release);
                    self.reprt_handler.store(true, Ordering::Release);
                    spin_loop();
                    return None;
                }

                self.list_core
                    .packet_core
                    .ready_bitmap
                    .fetch_and(masking_on, Ordering::Release);

                self.list_core
                    .store_packet_exec_index(index as usize, Ordering::Release);
                self.list_core
                    .store_packet_exec_status(true, Ordering::Release);
                self.reprt_handler.store(true, Ordering::Release);
                spin_loop();
                return Some(());
            } else {
                return None;
            }
        } else {
            Some(())
        }
    }
}
