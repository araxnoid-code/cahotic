use std::{
    collections::VecDeque,
    fmt::Debug,
    hint::spin_loop,
    os::fd,
    ptr::{null, null_mut},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use crate::{
    ExecTask, ListCore, OutputTrait, PollWaiting, TaskDependencies, TaskDependenciesCore,
    TaskTrait, TaskWithDependenciesTrait, WaitingTask,
};

pub struct ThreadUnit<F, FD, O>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send + Debug,
{
    // thread
    // // unique
    pub(crate) id: usize,
    // // drop-stack
    pub(crate) drop_queue: VecDeque<*mut WaitingTask<F, FD, O>>,

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
    O: 'static + OutputTrait + Send + Debug,
{
    pub fn running(&mut self) {
        // main loop
        loop {
            if self.join_flag.load(Ordering::Acquire) {
                break;
            }

            // is primary list empty?
            let is_reprt = self.reprt_handler.swap(false, Ordering::AcqRel);
            if is_reprt {
                if self.list_core.is_primary_list_empty() {
                    if let Err(_) = self.list_core.swap_to_primary() {}
                }

                (*self.reprt_handler).store(true, Ordering::Release);
                spin_loop();
            }

            if let Some(drop_waiting_task) = self.drop_queue.pop_front() {
                if let Err(waiting_task) = self.list_core.drop_execute(drop_waiting_task) {
                    self.drop_queue.push_back(waiting_task);
                } else {
                    self.done_task.fetch_add(1, Ordering::Release);
                }
            }

            let task = if let Ok(task) = self.list_core.get_waiting_task_from_primary_stack() {
                task
            } else {
                spin_loop();
                continue;
            };

            // execute
            unsafe {
                if let ExecTask::DropPoll(_)
                | ExecTask::DropDependencies(_)
                | ExecTask::DropPollAfter(_, _)
                | ExecTask::DropDependenciesAfter(_, _) = (*task).task
                {
                    if let Err(waiting_task) = self.list_core.drop_execute(task) {
                        self.drop_queue.push_back(waiting_task);
                    } else {
                        self.done_task.fetch_add(1, Ordering::Release);
                    }

                    spin_loop();
                } else {
                    let box_task = Box::from_raw(task);
                    let output = match &box_task.task {
                        ExecTask::Task(f) => f.execute(),
                        ExecTask::TaskWithDependencies(f) => {
                            f.execute(box_task.output_dependencies_ptr.expect(
                                "Thread Error, function need dependencies but dependencies is None",
                            ))
                        }
                        ExecTask::Output(_) => panic!(),
                        ExecTask::DropPoll(_) => panic!(),
                        ExecTask::DropDependencies(_) => panic!(),
                        ExecTask::DropPollAfter(_, _) => panic!(),
                        ExecTask::DropDependenciesAfter(_, _) => panic!(),
                        ExecTask::None => panic!(),
                    };

                    let output = Box::into_raw(Box::new(output));
                    box_task.return_ptr.store(output, Ordering::Release);

                    let _ = self.dependencies_handler(box_task);

                    // update counter
                    self.done_task.fetch_add(1, Ordering::SeqCst);
                    spin_loop();
                }
            }
        }
    }

    fn dependencies_handler(&self, task: Box<WaitingTask<F, FD, O>>) -> Option<()> {
        if let Some(dependencies_core_ptr) = task.dependencies_core_ptr {
            // update counter
            let counter = dependencies_core_ptr
                .counter
                .fetch_sub(1, Ordering::Release);

            if counter - 1 != 0 {
                return None;
            }

            // update done flag
            dependencies_core_ptr.done.store(true, Ordering::Release);

            let check_task = dependencies_core_ptr.start.load(Ordering::Acquire);
            if !check_task.is_null() {
                // CAS RETRY LOOP
                let start_waiting_task = loop {
                    let status = dependencies_core_ptr.start.compare_exchange(
                        dependencies_core_ptr.start.load(Ordering::Acquire),
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

                let end_waiting_task = dependencies_core_ptr
                    .end
                    .swap(null_mut(), Ordering::Acquire);

                self.list_core
                    .task_from_dependencies(start_waiting_task, end_waiting_task);
            }

            dependencies_core_ptr
                .drop_ready
                .store(true, Ordering::Release);
        }

        drop(task);

        Some(())
    }
}
