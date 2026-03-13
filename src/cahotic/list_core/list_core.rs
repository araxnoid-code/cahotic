use std::{
    hint::spin_loop,
    ptr::{self, null_mut},
    sync::{
        Arc,
        atomic::{AtomicPtr, AtomicU64, Ordering},
    },
};

use crate::{
    ExecTask, OutputTrait, PoolOutput, TaskDependencies, TaskDependenciesCore,
    TaskDependenciesTrait, TaskTrait, TaskWithDependenciesTrait, WaitingTask, cahotic::task,
};

pub struct ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // primary Stack
    pub(crate) id_counter: AtomicU64,
    pub(crate) start: AtomicPtr<WaitingTask<F, FD, O>>,
    pub(crate) end: AtomicPtr<WaitingTask<F, FD, O>>,

    // handler
    pub(crate) in_task: Arc<AtomicU64>,

    // Swap Stack
    pub(crate) swap_start: AtomicPtr<WaitingTask<F, FD, O>>,
    pub(crate) swap_end: AtomicPtr<WaitingTask<F, FD, O>>,
}

impl<F, FD, O> ListCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> ListCore<F, FD, O> {
        Self {
            // primary Stack
            id_counter: AtomicU64::new(0),
            start: AtomicPtr::new(ptr::null_mut()),
            end: AtomicPtr::new(ptr::null_mut()),

            // handler
            in_task: Arc::new(AtomicU64::new(0)),

            // Swap Stack
            swap_start: AtomicPtr::new(ptr::null_mut()),
            swap_end: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn get_waiting_task_from_primary_stack(&self) -> Result<*mut WaitingTask<F, FD, O>, &str> {
        let start_waiting_task = self.start.load(Ordering::Acquire);

        unsafe {
            let waiting_task = self.end.load(Ordering::Acquire);
            if waiting_task.is_null() {
                return Err("Primary list empty");
            }

            let next = (*waiting_task).next.load(Ordering::Acquire);
            if next.is_null() {
                if waiting_task != start_waiting_task {
                    return Err("Failed get task");
                }
            }

            let status =
                self.end
                    .compare_exchange(waiting_task, next, Ordering::AcqRel, Ordering::Acquire);

            let task = if let Ok(task) = status {
                task
            } else {
                return Err("Failed get task");
            };

            Ok(task)
        }
    }

    pub fn is_primary_list_empty(&self) -> bool {
        self.end.load(Ordering::Acquire).is_null()
    }

    pub fn is_swap_list_empty(&self) -> bool {
        self.swap_end.load(Ordering::Acquire).is_null()
    }

    pub fn swap_to_primary(&self) -> Result<(), &str> {
        // if !self.end.load(Ordering::Acquire).is_null() {
        //     return Err("PRIMARY LIST NOT EMPTY");
        // }

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
}
