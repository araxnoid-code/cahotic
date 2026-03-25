use std::{
    collections::VecDeque,
    hint::spin_loop,
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    thread::{JoinHandle, spawn},
};

use crate::{ListCore, OutputTrait, SchedulerTrait, TaskTrait, ThreadUnit};

pub struct ThreadPoolCore<F, FD, O, const N: usize, const PN: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // main thread pool
    pub(crate) pool: Vec<JoinHandle<()>>,

    // handler
    pub(crate) done_task: Arc<AtomicU64>,
    pub(crate) join_flag: Arc<AtomicBool>,

    // list core
    list_core: Arc<ListCore<F, FD, O, PN>>,
}

impl<F, FD, O, const N: usize, const PN: usize> ThreadPoolCore<F, FD, O, N, PN>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FD: SchedulerTrait<O> + Send + 'static + Sync,
    O: OutputTrait + Send + Sync,
{
    pub fn init(list_core: Arc<ListCore<F, FD, O, PN>>) -> ThreadPoolCore<F, FD, O, N, PN> {
        // handler
        let join_flag = Arc::new(AtomicBool::new(false));
        let done_task = Arc::new(AtomicU64::new(0));

        // pool
        let mut pool = Vec::with_capacity(N);

        // block
        let block = Arc::new(AtomicBool::new(false));

        for id in 0..N {
            // clone
            let join_flag_clone = join_flag.clone();
            let done_task_clone = done_task.clone();
            let list_core_clone = list_core.clone();
            let block_clone = block.clone();

            let spawn = spawn(move || {
                let mut thread_unit = ThreadUnit {
                    _id: id,
                    scheduling_queue: VecDeque::with_capacity(1024),
                    done_task: done_task_clone,
                    join_flag: join_flag_clone,
                    list_core: list_core_clone,
                    packet_drop_queue: VecDeque::with_capacity(64),
                    exec_packet_idx: 64,
                    masking_packet_idx: 64,
                };

                while !block_clone.load(Ordering::Acquire) {
                    spin_loop();
                }

                // running
                thread_unit.running_packet();
            });

            // saving
            pool.push(spawn);
        }

        block.store(true, Ordering::Release);

        Self {
            done_task,
            join_flag,
            list_core,
            pool,
        }
    }

    pub fn join(self) {
        unsafe {
            // clean
            // check, all task done
            self.list_core.submit_packet();
            loop {
                if self.list_core.in_task.load(Ordering::Acquire)
                    <= self.done_task.load(Ordering::Acquire)
                {
                    break;
                }
                spin_loop();
            }

            // clean packet
            let packet_list_ptr = self
                .list_core
                .packet_core
                .packet_list
                .swap(null_mut(), Ordering::Acquire);
            for packet in &*packet_list_ptr {
                drop(Box::from_raw(
                    packet.done_counter as *const AtomicUsize as *mut AtomicUsize,
                ));
            }
            drop(Box::from_raw(packet_list_ptr));

            // join
            self.join_flag.store(true, Ordering::Release);
            for join_handle in self.pool {
                join_handle.join().unwrap();
            }
        }
    }
}
