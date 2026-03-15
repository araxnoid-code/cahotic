use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    ExecTask, ListCore, OutputTrait, PoolOutput, TaskDependenciesCore, TaskTrait,
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
            if let ExecTask::DropPool(drop_sch) = &(*waiting_task_ptr).task {
                if let Some(_) = drop_sch.pool_wait.get() {
                    // drop pool
                    let output = drop_sch
                        .pool_wait
                        .output
                        .data_ptr
                        .swap(null_mut(), Ordering::AcqRel);

                    drop(Box::from_raw(output));

                    drop(Box::from_raw(
                        drop_sch.pool_wait.output.data_ptr as *const AtomicPtr<O>
                            as *mut AtomicPtr<O>,
                    ));

                    // drop task
                    let task = Box::from_raw(waiting_task_ptr);

                    drop(Box::from_raw(
                        task.return_ptr as *const AtomicPtr<O> as *mut AtomicPtr<O>,
                    ));

                    Ok(())
                } else {
                    Err(waiting_task_ptr)
                }
            } else if let ExecTask::DropDependencies(dependencies) = &(*waiting_task_ptr).task {
                if dependencies
                    .task_dependencies_ptr
                    .drop_ready
                    .load(Ordering::Acquire)
                {
                    drop(Box::from_raw(
                        (dependencies).task_dependencies_ptr
                            as *const TaskDependenciesCore<F, FD, O>
                            as *mut TaskDependenciesCore<F, FD, O>,
                    ));

                    let waiting_list = Box::from_raw(
                        (dependencies).waiting_list as *const Vec<PoolOutput<O>>
                            as *mut Vec<PoolOutput<O>>,
                    );

                    for waiting in waiting_list.iter() {
                        let output = waiting.data_ptr.swap(null_mut(), Ordering::AcqRel);
                        drop(Box::from_raw(output));
                        drop(Box::from_raw(
                            waiting.data_ptr as *const AtomicPtr<O> as *mut AtomicPtr<O>,
                        ))
                    }

                    // drop task
                    let task = Box::from_raw(waiting_task_ptr);

                    drop(Box::from_raw(
                        task.return_ptr as *const AtomicPtr<O> as *mut AtomicPtr<O>,
                    ));

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
