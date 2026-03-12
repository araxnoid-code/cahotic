use std::{
    hint::spin_loop,
    ptr::{self, null_mut},
    sync::{
        Arc,
        atomic::{AtomicPtr, AtomicU64, Ordering},
    },
};

use crate::{
    ExecTask, OutputTrait, PoolTask, TaskDependenciesCore, TaskTrait, TaskWithDependenciesTrait,
    WaitingTask, cahotic::task,
};

pub struct ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // primary Stack
    id_counter: AtomicU64,
    start: AtomicPtr<WaitingTask<F, FD, O>>,
    end: AtomicPtr<WaitingTask<F, FD, O>>,

    // handler
    pub(crate) in_task: Arc<AtomicU64>,

    // Swap Stack
    swap_start: AtomicPtr<WaitingTask<F, FD, O>>,
    swap_end: AtomicPtr<WaitingTask<F, FD, O>>,
}

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn get_waiting_task_from_primary_stack(&self) -> Result<*mut WaitingTask<F, FD, O>, &str> {
        let start_waiting_task = self.start.load(Ordering::Acquire);

        unsafe {
            let task = loop {
                let waiting_task = self.end.load(Ordering::Acquire);
                if waiting_task.is_null() {
                    return Err("Primary list empty");
                }

                let next = (*waiting_task).next.load(Ordering::Acquire);
                if next.is_null() {
                    while waiting_task != start_waiting_task {
                        spin_loop();
                    }
                }

                let status = self.end.compare_exchange(
                    waiting_task,
                    next,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );

                if let Ok(task) = status {
                    break task;
                }
            };

            Ok(task)
        }
    }

    pub fn is_primary_list_empty(&self) -> bool {
        self.end.load(Ordering::Acquire).is_null()
    }

    pub fn swap_to_primary(&self) -> Result<(), &str> {
        if !self.end.load(Ordering::Acquire).is_null() {
            return Err("PRIMARY LIST NOT EMPTY");
        }

        let swap_end = self.swap_end.swap(null_mut(), Ordering::AcqRel);
        if !swap_end.is_null() {
            let swap_start = self.swap_start.swap(null_mut(), Ordering::AcqRel);
            self.start.store(swap_start, Ordering::Release);
            self.end.store(swap_end, Ordering::Release);
            Ok(())
        } else {
            Err("SWAP LIST EMPTY")
        }
    }

    pub fn spawn_task(&self, task: F) -> PoolTask<O> {
        // main thread only focus in swap queue, base on swap start
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
            task_dependencies_core_ptr: Box::leak(Box::new(TaskDependenciesCore::blank())),
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

        PoolTask {
            data_ptr: return_ptr,
        }
    }
}
