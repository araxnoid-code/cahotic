use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    DropSchedule, ExecTask, ListCore, OutputTrait, PoolOutput, PoolWait, TaskDependenciesCore,
    TaskTrait, TaskWithDependenciesTrait, WaitingTask, cahotic::list_core::drop::drop_sch,
};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) fn drop_pool_sch(
        &self,
        waiting_task_ptr: *mut WaitingTask<F, FD, O>,
    ) -> Result<(), *mut WaitingTask<F, FD, O>> {
        unsafe {
            if let ExecTask::Drop(drop_sch) = &(*waiting_task_ptr).task {
                if let Some(_) = drop_sch.pool_wait.get() {
                    // drop pool
                    drop(Box::from_raw(
                        drop_sch.pool_wait.output.data_ptr.load(Ordering::Acquire),
                    ));
                    drop(Box::from_raw(
                        drop_sch.pool_wait.output_dependencies_ptr as *const Vec<PoolOutput<O>>
                            as *mut Vec<PoolOutput<O>>,
                    ));
                    drop(Box::from_raw(
                        drop_sch.pool_wait.dependencies_core_ptr
                            as *const TaskDependenciesCore<F, FD, O>
                            as *mut TaskDependenciesCore<F, FD, O>,
                    ));

                    // drop task
                    let task = Box::from_raw(waiting_task_ptr);
                    drop(Box::from_raw(
                        task.dependencies_core_ptr as *const TaskDependenciesCore<F, FD, O>
                            as *mut TaskDependenciesCore<F, FD, O>,
                    ));

                    drop(Box::from_raw(
                        task.output_dependencies_ptr as *const Vec<PoolOutput<O>>
                            as *mut Vec<PoolOutput<O>>,
                    ));

                    drop(Box::from_raw(
                        task.return_ptr as *const AtomicPtr<O> as *mut AtomicPtr<O>,
                    ));

                    Ok(())
                } else {
                    Err(waiting_task_ptr)
                }
            } else {
                panic!("imposible")
            }
        }
    }

    pub fn drop_pool(&self, pool_wait: PoolWait<F, FD, O>) {
        if let Some(_) = pool_wait.get() {
            unsafe {
                drop(Box::from_raw(
                    pool_wait.output.data_ptr.load(Ordering::Acquire),
                ));
                drop(Box::from_raw(
                    pool_wait.output_dependencies_ptr as *const Vec<PoolOutput<O>>
                        as *mut Vec<PoolOutput<O>>,
                ));
                drop(Box::from_raw(
                    pool_wait.dependencies_core_ptr as *const TaskDependenciesCore<F, FD, O>
                        as *mut TaskDependenciesCore<F, FD, O>,
                ));
            }
        } else {
            let drop_sch = DropSchedule { pool_wait };
            // update in_task handler
            self.in_task.fetch_add(1, Ordering::Release);
            // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
            // dependencies
            let dependencies_core_ptr = Box::leak(Box::new(TaskDependenciesCore::blank()));
            let output_dependencies_ptr = Box::leak(Box::new(Vec::new()));
            // create waiting task
            let waiting_task = WaitingTask {
                id: self.id_counter.fetch_add(1, Ordering::Release),
                task: ExecTask::Drop(drop_sch),
                next: AtomicPtr::new(null_mut()),
                return_ptr,
                dependencies_core_ptr,
                output_dependencies_ptr,
            };

            let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

            // swap start with new waiting task
            let pre_start_task = self.swap_start.swap(waiting_task_ptr, Ordering::AcqRel);
            unsafe {
                (*pre_start_task)
                    .next
                    .store(waiting_task_ptr, Ordering::Release);
            }
        }
    }
}
