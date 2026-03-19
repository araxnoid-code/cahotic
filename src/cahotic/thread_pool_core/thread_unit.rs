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
    ExecTask, ListCore, OutputTrait, SchedulerTrait, SchedulerVec, TaskTrait, WaitingTask,
};

pub struct ThreadUnit<F, FS, O>
where
    F: TaskTrait<O> + 'static + Send,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
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
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
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
                if let ExecTask::DropPoll(_, _) = (*task).task {
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
                    match box_task.task {
                        ExecTask::Task(f, done_arena_counter) => {
                            let output = Box::into_raw(Box::new(f.execute()));
                            box_task
                                .return_ptr
                                .unwrap()
                                .store(output, Ordering::Release);

                            done_arena_counter.fetch_sub(1, Ordering::Release);
                        }
                        ExecTask::Scheduling(_, _, _, _) => panic!(),
                        ExecTask::Output(_) => panic!(),
                        ExecTask::DropPoll(_, _) => panic!(),
                        ExecTask::None => panic!(),
                    };

                    // update counter
                    self.done_task.fetch_add(1, Ordering::SeqCst);
                    spin_loop();
                }
            }
        }
    }
}
