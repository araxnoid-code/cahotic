use std::{
    collections::VecDeque,
    fmt::Debug,
    hint::spin_loop,
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use crate::{
    ExecTask, ListCore, OutputTrait, SchedulerVec, TaskTrait, TaskWithDependenciesTrait,
    WaitingTask,
};

pub struct ThreadUnit<F, FS, O>
where
    F: TaskTrait<O> + 'static + Send,
    FS: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send + Debug,
{
    // thread
    // // unique
    pub(crate) id: usize,
    // // drop-stack
    pub(crate) scheduling_queue: VecDeque<*mut WaitingTask<F, FS, O>>,

    // share
    // // thread_pool
    pub(crate) join_flag: Arc<AtomicBool>,
    pub(crate) done_task: Arc<AtomicU64>,
    pub(crate) reprt_handler: Arc<AtomicBool>,

    // // list core
    pub(crate) list_core: Arc<ListCore<F, FS, O>>,
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

            if let Some(schedul_waiting_task) = self.scheduling_queue.pop_front() {
                if let Some(waiting_task) = self.list_core.scheduling_handler(schedul_waiting_task)
                {
                    unsafe {
                        let box_task = Box::from_raw(waiting_task);

                        if let ExecTask::Scheduling(f, waiting_poll, _, done_arena_counter) =
                            box_task.task
                        {
                            let output = Box::into_raw(Box::new(
                                f.execute(SchedulerVec { vec: waiting_poll }),
                            ));

                            box_task
                                .return_ptr
                                .unwrap()
                                .store(output, Ordering::Release);

                            done_arena_counter.fetch_sub(1, Ordering::Release);
                            self.done_task.fetch_add(1, Ordering::SeqCst);
                            spin_loop();
                        }
                    }
                } else {
                    self.scheduling_queue.push_back(schedul_waiting_task);
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
                if let ExecTask::DropPoll(_, _)
                | ExecTask::DropDependencies(_, _)
                | ExecTask::DropPollAfter(_, _)
                | ExecTask::DropDependenciesAfter(_, _) = (*task).task
                {
                    if let Err(waiting_task) = self.list_core.drop_execute(task) {
                        self.scheduling_queue.push_back(waiting_task);
                    } else {
                        self.done_task.fetch_add(1, Ordering::Release);
                    }

                    spin_loop();
                } else if let ExecTask::Scheduling(_, _, _, _) = (*task).task {
                    if let Some(waiting_task) = self.list_core.scheduling_handler(task) {
                        let box_task = Box::from_raw(waiting_task);

                        if let ExecTask::Scheduling(f, waiting_poll, _, done_arena_counter) =
                            box_task.task
                        {
                            let output = Box::into_raw(Box::new(
                                f.execute(SchedulerVec { vec: waiting_poll }),
                            ));

                            box_task
                                .return_ptr
                                .unwrap()
                                .store(output, Ordering::Release);

                            done_arena_counter.fetch_sub(1, Ordering::Release);
                            self.done_task.fetch_add(1, Ordering::SeqCst);
                            spin_loop();
                        }
                    } else {
                        self.scheduling_queue.push_back(task);
                    }
                } else {
                    let box_task = Box::from_raw(task);
                    let output = match box_task.task {
                        ExecTask::Task(f, done_arena_counter) => {
                            let output = Box::into_raw(Box::new(f.execute()));
                            box_task
                                .return_ptr
                                .unwrap()
                                .store(output, Ordering::Release);

                            done_arena_counter.fetch_sub(1, Ordering::Release);
                        }
                        ExecTask::Scheduling(_, _, _, _) => panic!(),
                        ExecTask::TaskWithDependencies(f, done_arena_counter) => {
                            // let output = Box::into_raw(Box::new(f.execute(box_task.output_dependencies_ptr.expect(
                            //     "Thread Error, function need dependencies but dependencies is None",
                            // ))));

                            // box_task
                            //     .return_ptr
                            //     .unwrap()
                            //     .store(output, Ordering::Release);

                            // done_arena_counter.fetch_sub(1, Ordering::Release);
                        }
                        ExecTask::Output(_) => panic!(),
                        ExecTask::DropPoll(_, _) => panic!(),
                        ExecTask::DropDependencies(_, _) => panic!(),
                        ExecTask::DropPollAfter(_, _) => panic!(),
                        ExecTask::DropDependenciesAfter(_, _) => panic!(),
                        ExecTask::None => panic!(),
                    };

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
