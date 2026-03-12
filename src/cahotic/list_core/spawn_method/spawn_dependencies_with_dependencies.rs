use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    ArrTaskDependenciesWithDependenciesTrait, ExecTask, ListCore, OutputTrait, PoolWait,
    TaskDependencies, TaskDependenciesCore, TaskTrait, TaskWithDependenciesTrait, WaitingTask,
};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn spawn_task_dependencies_with_dependencies<D, const NF: usize>(
        &self,
        dependencies: D,
        with_dependencies: &TaskDependencies<F, FD, O>,
    ) -> TaskDependencies<F, FD, O>
    where
        D: ArrTaskDependenciesWithDependenciesTrait<FD, O, NF>,
    {
        // create dependencies
        let task_dependencies_core_ptr: &'static TaskDependenciesCore<F, FD, O> =
            Box::leak(Box::new(TaskDependenciesCore::init(NF)));

        // output
        let mut waiting_output = Vec::with_capacity(NF);

        // task_dependencies
        for task in dependencies.task_list() {
            // create waiting task
            if task.is_with_dependencies() {
                let waiting_task = self.spawn_task_with_dependencies(
                    task,
                    with_dependencies,
                    Some(task_dependencies_core_ptr),
                );
                waiting_output.push(waiting_task);
            } else {
                let waiting_task = self.spawn_task_fd(task, Some(task_dependencies_core_ptr));
                waiting_output.push(waiting_task);
            }
        }

        let waiting_output_leak: &'static mut Vec<PoolWait<O>> =
            Box::leak(Box::new(waiting_output));
        TaskDependencies {
            waiting_list: waiting_output_leak,
            task_dependencies_ptr: task_dependencies_core_ptr,
        }
    }

    pub fn spawn_task_fd(
        &self,
        task: FD,
        task_dependencies_core_ptr: Option<&'static TaskDependenciesCore<F, FD, O>>,
    ) -> PoolWait<O> {
        // main thread only focus in swap queue, base on swap start
        // update in_task handler
        self.in_task.fetch_add(1, Ordering::SeqCst);
        // create return_ptr
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task: ExecTask::TaskWithDependencies(task),
            next: AtomicPtr::new(null_mut()),
            waiting_return_ptr: return_ptr,
            task_dependencies_core_ptr: if let Some(ptr) = task_dependencies_core_ptr {
                ptr
            } else {
                Box::leak(Box::new(TaskDependenciesCore::blank()))
            },
            task_dependencies_ptr: Box::leak(Box::new(Vec::new())),
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

        PoolWait {
            data_ptr: return_ptr,
        }
    }
}
