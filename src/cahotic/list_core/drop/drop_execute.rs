use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{ExecTask, ListCore, OutputTrait, PollWaiting, SchedulerTrait, TaskTrait, WaitingTask};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) fn drop_execute(
        &self,
        waiting_task_ptr: *mut WaitingTask<F, FD, O>,
    ) -> Result<(), *mut WaitingTask<F, FD, O>> {
        unsafe {
            if let ExecTask::DropPoll(poll_waiting, _) = &(*waiting_task_ptr).task {
                if let Some(_) = poll_waiting.get() {
                    // drop pool
                    let output = poll_waiting.data_ptr.swap(null_mut(), Ordering::AcqRel);

                    drop(Box::from_raw(output));

                    drop(Box::from_raw(
                        poll_waiting.data_ptr as *const AtomicPtr<O> as *mut AtomicPtr<O>,
                    ));

                    drop(Box::from_raw(
                        poll_waiting.drop_after_caounter as *const AtomicUsize as *mut AtomicUsize,
                    ));

                    // drop task
                    drop(Box::from_raw(waiting_task_ptr));
                    Ok(())
                } else {
                    Err(waiting_task_ptr)
                }
            } else {
                panic!()
            }
        }
    }
}
