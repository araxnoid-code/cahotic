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
    ExecTask, ListCore, OutputTrait, PoolOutput, TaskTrait, TaskWithDependenciesTrait, WaitingTask,
};

pub struct ThreadUnit<F, FD, O>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
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
            if self.join_flag.load(Ordering::Acquire) {
                break;
            }

            // is primary list empty?
            let is_reprt = self.reprt_handler.swap(false, Ordering::AcqRel);
            if is_reprt {
                if self.list_core.is_primary_list_empty() {
                    if let Err(_) = self.list_core.swap_to_primary() {
                        //
                    }
                }

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

                let _ = self.dependencies_handler(task);

                // update counter
                self.done_task.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    fn dependencies_handler(&self, task: Box<WaitingTask<F, FD, O>>) -> Option<()> {
        if task.task_dependencies_core_ptr.status {
            // update counter
            let counter = task
                .task_dependencies_core_ptr
                .counter
                .fetch_sub(1, Ordering::Release);

            if counter - 1 != 0 {
                return None;
            }

            // update done flag
            task.task_dependencies_core_ptr
                .done
                .store(true, Ordering::Release);

            let check_task = task
                .task_dependencies_core_ptr
                .start
                .load(Ordering::Acquire);
            if !check_task.is_null() {
                // CAS RETRY LOOP
                let start_waiting_task = loop {
                    let status = task.task_dependencies_core_ptr.start.compare_exchange(
                        task.task_dependencies_core_ptr
                            .start
                            .load(Ordering::Acquire),
                        null_mut(),
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    );

                    if let Ok(waiting_task) = status {
                        break waiting_task;
                    } else {
                        spin_loop();
                        continue;
                    }
                };

                let end_waiting_task = task
                    .task_dependencies_core_ptr
                    .end
                    .swap(null_mut(), Ordering::Acquire);

                self.list_core
                    .task_from_dependencies(start_waiting_task, end_waiting_task);
            }
        }

        drop(task);

        Some(())
    }
}
