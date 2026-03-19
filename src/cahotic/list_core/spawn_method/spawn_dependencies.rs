use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{
    ExecTask, ListCore, OutputTrait, PollWaiting, TaskDependencies, TaskDependenciesCore,
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
            self.drop_arena.add_current_done_counter_ptr(1);
            self.in_task.fetch_add(1, Ordering::Release);
            // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            // create waiting task
            let waiting_task = WaitingTask {
                id: self.id_counter.fetch_add(1, Ordering::Release),
                task: ExecTask::Task(task, self.drop_arena.get_current_done_counter_ptr()),
                next: AtomicPtr::new(null_mut()),
                return_ptr: Some(return_ptr),
                dependencies_core_ptr: Some(task_dependencies_core_ptr),
                output_dependencies_ptr: None,
            };

            let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

            // swap start with new waiting task
            let pre_start_task = self.swap_start.swap(waiting_task_ptr, Ordering::AcqRel);
            unsafe {
                (*pre_start_task)
                    .next
                    .store(waiting_task_ptr, Ordering::Release);
            }

            self.drop_pool(PollWaiting {
                data_ptr: return_ptr,
                drop_after_caounter: Box::leak(Box::new(AtomicUsize::new(0))),
            });

            waiting_output.push(PollWaiting {
                data_ptr: return_ptr,
                drop_after_caounter: Box::leak(Box::new(AtomicUsize::new(0))),
            });
        }

        let waiting_output_leak: &'static mut Vec<PollWaiting<O>> =
            Box::leak(Box::new(waiting_output));

        self.drop_dependencies(TaskDependencies {
            waiting_list: waiting_output_leak,
            task_dependencies_ptr: task_dependencies_core_ptr,
        });

        TaskDependencies {
            waiting_list: waiting_output_leak,
            task_dependencies_ptr: task_dependencies_core_ptr,
        }
    }
}
