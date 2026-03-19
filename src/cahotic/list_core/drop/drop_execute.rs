use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{
    ExecTask, ListCore, OutputTrait, PollWaiting, TaskDependenciesCore, TaskTrait,
    TaskWithDependenciesTrait, WaitingTask,
};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) fn drop_execute(
        &self,
        waiting_task_ptr: *mut WaitingTask<F, FD, O>,
    ) -> Result<(), *mut WaitingTask<F, FD, O>> {
        unsafe {
            if let ExecTask::DropPoll(poll_waiting, arena_done_counter) = &(*waiting_task_ptr).task
            {
                if let Some(_) = poll_waiting.get() {
                    println!("drop task {}", (*waiting_task_ptr).id);
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
            } else if let ExecTask::DropDependencies(dependencies, arena_done_counter) =
                &(*waiting_task_ptr).task
            {
                if dependencies
                    .task_dependencies_ptr
                    .drop_ready
                    .load(Ordering::Acquire)
                {
                    println!("drop dependencies {}", (*waiting_task_ptr).id);
                    drop(Box::from_raw(
                        (dependencies).task_dependencies_ptr
                            as *const TaskDependenciesCore<F, FD, O>
                            as *mut TaskDependenciesCore<F, FD, O>,
                    ));

                    drop(Box::from_raw(
                        (dependencies).waiting_list as *const Vec<PollWaiting<O>>
                            as *mut Vec<PollWaiting<O>>,
                    ));
                    // for waiting in waiting_list.iter() {
                    //     let output = waiting.data_ptr.swap(null_mut(), Ordering::AcqRel);
                    //     drop(Box::from_raw(output));
                    //     drop(Box::from_raw(
                    //         waiting.data_ptr as *const AtomicPtr<O> as *mut AtomicPtr<O>,
                    //     ))
                    // }

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
