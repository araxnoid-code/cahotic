use std::{
    ptr::{self, null_mut},
    sync::{
        Arc,
        atomic::{AtomicPtr, AtomicU64, AtomicUsize, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use crate::{DropArena, ExecTask, OutputTrait, PacketCore, SchedulerTrait, TaskTrait, WaitingTask};

pub struct ListCore<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // primary Stack
    pub(crate) id_counter: AtomicU64,
    pub(crate) start: AtomicPtr<WaitingTask<F, FS, O>>,
    pub(crate) end: AtomicPtr<WaitingTask<F, FS, O>>,

    // ArenaDrop
    pub(crate) swap_drop_arena: DropArena<F, FS, O>,

    // handler
    pub(crate) in_task: Arc<AtomicU64>,

    // Swap Stack
    pub(crate) swap_start: AtomicPtr<WaitingTask<F, FS, O>>,
    pub(crate) swap_end: AtomicPtr<WaitingTask<F, FS, O>>,
    // packet
    pub packet_core: PacketCore<F, FS, O, 8>,
}

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> ListCore<F, FD, O> {
        let waiting_task = WaitingTask {
            id: 0,
            task: ExecTask::None,
            next: AtomicPtr::new(null_mut()),
            return_ptr: None,
        };

        let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

        Self {
            // primary Stack
            id_counter: AtomicU64::new(1),
            start: AtomicPtr::new(ptr::null_mut()),
            end: AtomicPtr::new(ptr::null_mut()),

            // drop
            swap_drop_arena: DropArena::init(),

            // handler
            in_task: Arc::new(AtomicU64::new(0)),

            // Swap Stack
            swap_start: AtomicPtr::new(waiting_task_ptr),
            swap_end: AtomicPtr::new(waiting_task_ptr),
            // packet
            packet_core: PacketCore::init(),
        }
    }

    // pub fn drop_arena(&self) {
    //     let list = self.swap_drop_arena.drop_arena();
    //     if let Some((start, end, done_arena_counter)) = list {
    //         self.in_task.fetch_add(1, Ordering::Release);

    //         // create waiting task
    //         let waiting_task = WaitingTask {
    //             id: self.id_counter.fetch_add(1, Ordering::Release),
    //             task: ExecTask::DropArena(start, end, done_arena_counter),
    //             next: AtomicPtr::new(null_mut()),
    //             return_ptr: None,
    //         };

    //         let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));
    //         let pre_start_task = self.swap_start.swap(waiting_task_ptr, Ordering::AcqRel);
    //         unsafe {
    //             (*pre_start_task)
    //                 .next
    //                 .store(waiting_task_ptr, Ordering::Release);
    //         }
    //     }
    // }

    pub fn submit_packet(&self) {
        let _ = self.packet_core.submit_packet(&self.in_task);
    }

    pub fn get_waiting_task_from_primary_stack(
        &self,
    ) -> Result<*mut WaitingTask<F, FD, O>, &'static str> {
        unsafe {
            let waiting_task = self.end.load(Ordering::Acquire);
            if waiting_task.is_null() {
                return Err("Primary list empty");
            }
            let start_waiting_task = self.start.load(Ordering::Acquire);

            let next = (*waiting_task).next.load(Ordering::Acquire);
            if next.is_null() {
                if waiting_task != start_waiting_task {
                    return Err("Failed get task, not same");
                }
            }

            let status =
                self.end
                    .compare_exchange(waiting_task, next, Ordering::AcqRel, Ordering::Acquire);

            let task = if let Ok(task) = status {
                task
            } else {
                return Err("Failed get task");
            };

            Ok(task)
        }
    }

    pub fn is_primary_list_empty(&self) -> bool {
        self.end.load(Ordering::Acquire).is_null()
    }

    pub fn is_swap_list_empty(&self) -> bool {
        unsafe {
            (*self.swap_end.load(Ordering::Acquire))
                .next
                .load(Ordering::Acquire)
                .is_null()
        }
    }

    pub fn swap_to_primary(&self) -> Result<(), &str> {
        unsafe {
            let dummy_end = self.swap_end.load(Ordering::Acquire);

            let dummy_next = (*dummy_end).next.swap(null_mut(), Ordering::AcqRel);

            if !dummy_next.is_null() {
                let swap_start = self.swap_start.swap(dummy_end, Ordering::AcqRel);
                self.start.store(swap_start, Ordering::Release);
                self.end.store(dummy_next, Ordering::Release);
                Ok(())
            } else {
                Err("SWAP LIST EMPTY")
            }
        }
    }
}
