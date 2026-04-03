use crate::{ExecTask, OutputTrait, PacketCore, SchedulerTrait, TaskTrait, WaitingTask};
use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
};

//
pub enum EnqueueSlotStatus {
    Ok(usize),
    Waiting(usize),
}

pub enum DequeueStatus<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Ok(WaitingTask<F, FS, O>),
    Waiting(usize),
    None,
}
//

impl<F, FS, O, const PN: usize> PacketCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn load_head(&self, order: Ordering) -> usize {
        self.head.load(order)
    }

    pub fn enqueue(&self, task: F, id_counter: u64, in_task: &AtomicU64) {
        unsafe {
            let head = self.head.fetch_add(1, Ordering::Release) & 4095;

            let packet = &mut (*self.ring_buffer.load(Ordering::Relaxed))[head];
            while !packet.empty.load(Ordering::Acquire) {
                spin_loop();
            }

            // update handler
            in_task.fetch_add(1, Ordering::Release);
            // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            // create waiting task
            let waiting_task = WaitingTask {
                _id: id_counter,
                task: ExecTask::Task(task),
                return_ptr: Some(return_ptr),
                poll_child: vec![],
            };

            packet.task = Some(waiting_task);
            packet.empty.store(false, Ordering::Release);
        }
    }

    pub fn dequeue(&self) -> DequeueStatus<F, FS, O> {
        unsafe {
            let tail = self.tail.fetch_add(1, Ordering::Relaxed) & 4095;

            let packet = &mut (*self.ring_buffer.load(Ordering::Relaxed))[tail];
            while packet.empty.load(Ordering::Acquire) {
                return DequeueStatus::Waiting(tail);
            }

            if let Some(task) = packet.task.take() {
                return DequeueStatus::Ok(task);
            }

            packet.empty.store(true, Ordering::Release);

            return DequeueStatus::None;
        }
    }
}
