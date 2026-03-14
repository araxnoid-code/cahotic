use std::{
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
    ExecTask, ListCore, OutputTrait, PoolOutput, TaskDependenciesCore, TaskTrait,
    TaskWithDependenciesTrait, WaitingTask,
};

pub struct ThreadUnit<F, FD, O>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send + Debug,
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
    O: 'static + OutputTrait + Send + Debug,
{
    pub fn running(&self) {
        // main loop
        loop {
            // sleep(Duration::from_millis(25 * self.id as u64 + 1));
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
                spin_loop();
                continue;
            };

            // println!(
            //     "start prim: {:?}\nend prim: {:?}\nstart swap: {:?}\nend swap: {:?}",
            //     self.list_core.start,
            //     self.list_core.end,
            //     self.list_core.swap_start,
            //     self.list_core.swap_end
            // );

            // execute
            unsafe {
                if let ExecTask::Drop(_) = (*task).task {
                    if self.id == 2 {
                        println!("get task id {}", (*task).id);
                    }
                    if let Ok(_) = self.list_core.drop_pool_sch(task) {
                        // update counter
                        self.done_task.fetch_add(1, Ordering::SeqCst);
                    }
                    spin_loop();
                } else {
                    let box_task = Box::from_raw(task);
                    let output = match &box_task.task {
                        ExecTask::Task(f) => f.execute(),
                        ExecTask::TaskWithDependencies(f) => {
                            f.execute(box_task.output_dependencies_ptr)
                        }
                        ExecTask::Output(o) => {
                            panic!(
                                // "{}, what the fuck? => {:?} with task id {} then {}",
                                // self.id,
                                // o,
                                // self.id,
                                // task.is_null()
                            )
                        }
                        ExecTask::Drop(_) => panic!(),
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
        if task.dependencies_core_ptr.status {
            // update counter
            let counter = task
                .dependencies_core_ptr
                .counter
                .fetch_sub(1, Ordering::Release);

            if counter - 1 != 0 {
                return None;
            }

            // update done flag
            task.dependencies_core_ptr
                .done
                .store(true, Ordering::Release);

            let check_task = task.dependencies_core_ptr.start.load(Ordering::Acquire);
            if !check_task.is_null() {
                // CAS RETRY LOOP
                let start_waiting_task = loop {
                    let status = task.dependencies_core_ptr.start.compare_exchange(
                        task.dependencies_core_ptr.start.load(Ordering::Acquire),
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
                    .dependencies_core_ptr
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
