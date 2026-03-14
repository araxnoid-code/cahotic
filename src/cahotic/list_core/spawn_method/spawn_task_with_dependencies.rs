use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    ExecTask, ListCore, OutputTrait, PoolOutput, TaskDependencies, TaskDependenciesCore, TaskTrait,
    TaskWithDependenciesTrait, WaitingTask,
};

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn spawn_task_with_dependencies(
        &self,
        task: FD,
        dependencies: &TaskDependencies<F, FD, O>,
        task_dependencies_core_ptr: Option<&'static TaskDependenciesCore<F, FD, O>>,
    ) -> PoolOutput<O> {
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
            return_ptr,
            dependencies_core_ptr: if let Some(ptr) = task_dependencies_core_ptr {
                ptr
            } else {
                Box::leak(Box::new(TaskDependenciesCore::blank()))
            },
            output_dependencies_ptr: dependencies.waiting_list,
        };

        let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));
        // check depencies
        if !dependencies
            .task_dependencies_ptr
            .done
            .load(Ordering::SeqCst)
        {
            // insert into depencies waiting
            let status = dependencies.task_dependencies_ptr.start.compare_exchange(
                dependencies
                    .task_dependencies_ptr
                    .start
                    .load(Ordering::Acquire),
                waiting_task_ptr,
                Ordering::AcqRel,
                Ordering::Acquire,
            );

            if let Ok(prev_waiting_task) = status {
                if prev_waiting_task.is_null() {
                    // chek again
                    if !dependencies
                        .task_dependencies_ptr
                        .done
                        .load(Ordering::SeqCst)
                    {
                        if !prev_waiting_task.is_null() {
                            unsafe {
                                (*prev_waiting_task)
                                    .next
                                    .store(waiting_task_ptr, Ordering::Release);
                            }
                        } else {
                            dependencies
                                .task_dependencies_ptr
                                .end
                                .store(waiting_task_ptr, Ordering::Release);
                        }
                        dependencies
                            .task_dependencies_ptr
                            .len
                            .fetch_add(1, Ordering::SeqCst);
                    } else {
                        dependencies
                            .task_dependencies_ptr
                            .end
                            .store(null_mut(), Ordering::Release);
                        dependencies
                            .task_dependencies_ptr
                            .start
                            .store(null_mut(), Ordering::Release);
                        self.spawn_task_with_dependencies_normal(waiting_task_ptr);
                    };
                } else {
                    if !prev_waiting_task.is_null() {
                        unsafe {
                            (*prev_waiting_task)
                                .next
                                .store(waiting_task_ptr, Ordering::Release);
                        }
                    } else {
                        dependencies
                            .task_dependencies_ptr
                            .end
                            .store(waiting_task_ptr, Ordering::Release);
                    }

                    dependencies
                        .task_dependencies_ptr
                        .len
                        .fetch_add(1, Ordering::SeqCst);
                }
            } else {
                self.spawn_task_with_dependencies_normal(waiting_task_ptr);
            }
        } else {
            self.spawn_task_with_dependencies_normal(waiting_task_ptr);
        };

        PoolOutput {
            data_ptr: return_ptr,
        }
    }

    fn spawn_task_with_dependencies_normal(&self, waiting_task_ptr: *mut WaitingTask<F, FD, O>) {
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
    }
}
