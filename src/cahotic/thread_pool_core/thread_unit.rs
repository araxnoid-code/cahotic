use std::{
    hint::spin_loop,
    os::fd,
    ptr::{null, null_mut},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
};

use crate::{
    ExecTask, ListCore, OutputTrait, PoolWait, TaskTrait, TaskWithDependenciesTrait, WaitingTask,
};

pub struct ThreadUnit<F, FD, O>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // thread
    // // unique
    pub(crate) id: usize,
    // share
    // // thread_pool
    pub(crate) join_flag: Arc<AtomicBool>,
    pub(crate) done_task: Arc<AtomicU64>,
    pub(crate) reprt_handler: Arc<AtomicBool>,

    // // list core
    pub(crate) list_core: Arc<ListCore<F, FD, O>>,
}

impl<F, FD, O> ThreadUnit<F, FD, O>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn running(&self) {
        // main loop
        loop {
            // join flag?
            if self.join_flag.load(Ordering::Acquire) {
                break;
            }

            // is primary list empty?
            let is_reprt = self.reprt_handler.swap(false, Ordering::AcqRel);
            if self.list_core.is_primary_list_empty() && is_reprt {
                // now, this thread as representative thread
                // empty, swap waiting_task with swap list
                if let Err(_) = self.list_core.swap_to_primary() {
                    // this None, mean swap list empty or primary list not empty
                }
                // release representative thread
                (*self.reprt_handler).store(true, Ordering::Release);
                spin_loop();
            } else if is_reprt {
                (*self.reprt_handler).store(true, Ordering::Release);
                spin_loop();
            }

            let task = if let Ok(task) = self.list_core.get_waiting_task_from_primary_stack() {
                task
            } else {
                continue;
            };

            // execute
            unsafe {
                let task = Box::from_raw(task);

                let output = match &task.task {
                    ExecTask::Task(f) => f.execute(),
                    ExecTask::TaskWithDependencies(f) => f.execute(task.task_dependencies_ptr),
                    _ => panic!(),
                };

                let output = Box::into_raw(Box::new(output));
                task.waiting_return_ptr.store(output, Ordering::Release);

                // update counter
                self.done_task.fetch_add(1, Ordering::SeqCst);
            }
        }
    }
}
