use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle, sleep, spawn},
};

use crate::{ListCore, OutputTrait, TaskTrait, TaskWithDependenciesTrait, ThreadUnit, WaitingTask};

pub struct ThreadPoolCore<F, FD, O, const N: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // main thread pool
    pub(crate) pool: Arc<AtomicPtr<Vec<(Option<JoinHandle<()>>, Arc<ThreadUnit<F, FD, O>>)>>>,

    // handler
    pub(crate) reprt_handler: Arc<AtomicBool>,
    pub(crate) done_task: Arc<AtomicU64>,
    pub(crate) join_flag: Arc<AtomicBool>,

    // list core
    list_core: Arc<ListCore<F, FD, O>>,
}

impl<F, FD, O, const N: usize> ThreadPoolCore<F, FD, O, N>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: OutputTrait + Send,
{
    pub fn init(list_core: Arc<ListCore<F, FD, O>>) -> ThreadPoolCore<F, FD, O, N> {
        // handler
        let reprt_handler = Arc::new(AtomicBool::new(true));
        let join_flag = Arc::new(AtomicBool::new(false));
        let done_task = Arc::new(AtomicU64::new(0));

        // pool
        let pool = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Vec::with_capacity(
            N,
        )))));

        // block
        let block = Arc::new(AtomicBool::new(false));

        // MPSC
        let (tx, rx) = mpsc::channel();
        for id in 0..N {
            // MPSC
            let tx_clone = tx.clone();

            // clone
            let reprt_handler_clone = reprt_handler.clone();
            let join_flag_clone = join_flag.clone();
            let done_task_clone = done_task.clone();
            let list_core_clone = list_core.clone();
            let block_clone = block.clone();

            let spawn = spawn(move || {
                let thread_unit = Arc::new(ThreadUnit {
                    id,
                    done_task: done_task_clone,
                    join_flag: join_flag_clone,
                    list_core: list_core_clone,
                    reprt_handler: reprt_handler_clone,
                });

                tx_clone.send(thread_unit.clone()).unwrap();

                while !block_clone.load(Ordering::Acquire) {
                    spin_loop();
                }

                // running
                thread_unit.running();
            });

            // RX from MPSC
            let shared_thread = rx.recv().unwrap();
            // saving
            let threads_pool = pool.load(std::sync::atomic::Ordering::Acquire);
            unsafe {
                (*threads_pool).push((Some(spawn), shared_thread));
                pool.store(threads_pool, Ordering::Release);
            }
        }

        block.store(true, Ordering::Release);

        Self {
            done_task,
            join_flag,
            list_core,
            pool,
            reprt_handler,
        }
    }

    pub fn join(&self) {
        unsafe {
            // check, all task done
            loop {
                if self.list_core.in_task.load(Ordering::SeqCst)
                    <= self.done_task.load(Ordering::SeqCst)
                {
                    break;
                }
                spin_loop();
            }

            // join
            self.join_flag.store(true, Ordering::Release);
            for (join_handle, _) in (*self.pool.load(Ordering::Acquire)).iter_mut() {
                join_handle.take().unwrap().join().unwrap();
            }

            // for (_, thread) in (*self.pool.load(Ordering::Acquire)).iter_mut() {
            //     // thread.clean();
            // }

            // clean pool
            let pool_ptr = self.pool.swap(null_mut(), Ordering::AcqRel);
            drop(Box::from_raw(pool_ptr));
        }
    }
}
