use std::{
    array,
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    u64,
};

use crate::{
    ExecTask, OutputTrait, Packet, PollWaiting, Schedule, ScheduleTask, SchedulerTrait, TaskTrait,
    WaitingTask,
};

pub struct PacketCore<F, FS, O, const PN: usize>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub packet_list: AtomicPtr<[Packet<F, FS, O, PN>; 64]>,
    pub packet_len: usize,
    //
    pub empty_bitmap: AtomicU64,
    pub ready_bitmap: AtomicU64,
    //
    pub use_packet: AtomicU32,
    //
    pub exec_packet: AtomicUsize,
    pub exec_packet_handler: AtomicBool,
    pub masking: AtomicUsize,
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
            //
            empty_bitmap: AtomicU64::new(u64::MAX),
            ready_bitmap: AtomicU64::new(0),
            //
            use_packet: AtomicU32::new(64),
            //
            exec_packet_handler: AtomicBool::new(false),
            exec_packet: AtomicUsize::new(0),
            masking: AtomicUsize::new(64),
        }
    }

    fn set_use_packet(&self) {
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

            self.use_packet.store(
                self.empty_bitmap.load(Ordering::Acquire).trailing_zeros(),
                Ordering::Release,
            );
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
                    id: id_counter,
                    task: ExecTask::Task(task),
                    return_ptr: Some(return_ptr),
                };
                packet.task[idx] = Some(waiting_task);
                packet.drop[idx] = Some((return_ptr, None));

                PollWaiting {
                    data_ptr: return_ptr,
                }
            } else {
                let _ = self.submit_packet(in_task);
                self.add_task(task, id_counter, in_task)
            }
        }
    }

    pub fn add_schedule(
        &self,
        schedule: Schedule<F, FS, O>,
        id_counter: u64,
        in_task: &AtomicU64,
    ) -> PollWaiting<O> {
        unsafe {
            let mut use_packet_idx = self.use_packet.load(Ordering::Acquire) as usize;
            if use_packet_idx == 64 {
                self.set_use_packet();
                use_packet_idx = self.use_packet.load(Ordering::Acquire) as usize;
            }

            let packet = &mut (*self.packet_list.load(Ordering::Acquire))[use_packet_idx];
            let idx = packet.head.fetch_add(1, Ordering::Release);
            if idx + 1 < PN {
                // update in_task handler
                in_task.fetch_add(1, Ordering::Release);
                self.fetch_add_current_done_counter(
                    schedule.candidate_done_counter,
                    Ordering::Release,
                );
                schedule
                    .candidate_packet_idx
                    .store(use_packet_idx, Ordering::Release);

                // create return_ptr
                let return_ptr: &'static AtomicPtr<O> = schedule.return_ptr;

                let waiting_task = if let ScheduleTask::Task(task) = schedule.task {
                    WaitingTask {
                        id: id_counter,
                        task: ExecTask::<F, FS, O>::Task(task),
                        return_ptr: Some(return_ptr),
                    }
                } else if let (
                    ScheduleTask::Schedule(task),
                    Some(schedule_vec),
                    Some(candidate_packet),
                ) = (
                    schedule.task,
                    schedule.shcedule_vec,
                    schedule.candidate_packet_vec,
                ) {
                    let len = schedule_vec.len();
                    WaitingTask {
                        id: id_counter,
                        task: ExecTask::<F, FS, O>::Scheduling(
                            task,
                            schedule_vec,
                            if len == 0 { 0 } else { len - 1 },
                            use_packet_idx,
                            candidate_packet,
                        ),
                        return_ptr: Some(return_ptr),
                    }
                } else {
                    panic!()
                };

                // create waiting task
                packet.task[idx] = Some(waiting_task);
                packet.drop[idx] = Some((return_ptr, Some(schedule.candidate_packet_idx)));

                PollWaiting {
                    data_ptr: return_ptr,
                }
            } else {
                let _ = self.submit_packet(in_task);
                self.add_schedule(schedule, id_counter, in_task)
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
