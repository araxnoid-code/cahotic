use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    ExecTask, ListCore, OutputTrait, PoolOutput, TaskDependencies, TaskDependenciesCore,
    TaskDependenciesTrait, TaskTrait, TaskWithDependenciesTrait, WaitingTask,
};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn spawn_dependencies<D>(&self, dependencies: D) -> TaskDependencies<F, FD, O>
    where
        D: TaskDependenciesTrait<F, O>,
    {
        let task_list = dependencies.task_list();

        // create dependencies
        let task_dependencies_core_ptr: &'static TaskDependenciesCore<F, FD, O> =
            Box::leak(Box::new(TaskDependenciesCore::init(task_list.len())));

        // output
        let mut waiting_output = Vec::with_capacity(task_list.len());

        for task in task_list {
            // update in_task handler
            self.in_task.fetch_add(1, Ordering::SeqCst);
            // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            // create waiting task
            let waiting_task = WaitingTask {
                id: self.id_counter.fetch_add(1, Ordering::Release),
                task: ExecTask::Task(task),
                next: AtomicPtr::new(null_mut()),
                waiting_return_ptr: return_ptr,
                task_dependencies_core_ptr,
                task_dependencies_ptr: Box::leak(Box::new(vec![])),
            };

            let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

            // swap start with new waiting task
            let pre_start_task = self.swap_start.swap(waiting_task_ptr, Ordering::AcqRel);
            if !pre_start_task.is_null() {
                unsafe {
                    (*pre_start_task)
                        .next
                        .store(waiting_task_ptr, Ordering::Release);
                }
            } else {
                // saving end waiting task for spanning validation in thread pool later
                self.swap_end.store(waiting_task_ptr, Ordering::Release);
            }

            waiting_output.push(PoolOutput {
                data_ptr: return_ptr,
            });
        }

        let waiting_output_leak: &'static mut Vec<PoolOutput<O>> =
            Box::leak(Box::new(waiting_output));

        TaskDependencies {
            waiting_list: waiting_output_leak,
            task_dependencies_ptr: task_dependencies_core_ptr,
        }
    }
}
