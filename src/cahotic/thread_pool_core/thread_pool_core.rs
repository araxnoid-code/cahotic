use std::{
    array,
    hint::spin_loop,
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{JoinHandle, spawn},
};

use crate::{OutputTrait, PacketCore, SchedulerTrait, TaskTrait, ThreadUnit};

pub struct ThreadPoolCore<F, FD, O, const N: usize, const MAX_RING_BUFFER: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // main thread pool
    pub(crate) pool: [JoinHandle<()>; N],

    // handler
    pub(crate) done_task: Arc<AtomicU64>,
    pub(crate) join_flag: Arc<AtomicBool>,

    // list core
    packet_core: Arc<PacketCore<F, FD, O, MAX_RING_BUFFER>>,
}

impl<F, FD, O, const N: usize, const MAX_RING_BUFFER: usize>
    ThreadPoolCore<F, FD, O, N, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FD: SchedulerTrait<O> + Send + 'static + Sync,
    O: OutputTrait + Send + Sync,
{
    pub fn init(
        list_core: Arc<PacketCore<F, FD, O, MAX_RING_BUFFER>>,
    ) -> ThreadPoolCore<F, FD, O, N, MAX_RING_BUFFER> {
        // handler
        let join_flag = Arc::new(AtomicBool::new(false));
        let done_task = Arc::new(AtomicU64::new(0));

        // block
        let block = Arc::new(AtomicBool::new(false));

        // pool
        let pool = array::from_fn(|id| {
            // clone
            let join_flag_clone = join_flag.clone();
            let done_task_clone = done_task.clone();
            let list_core_clone = list_core.clone();
            let block_clone = block.clone();

            spawn(move || {
                let mut thread_unit = ThreadUnit {
                    _id: id,
                    break_counter: 0,
                    done_task: done_task_clone,
                    join_flag: join_flag_clone,
                    packet_core: list_core_clone,
                    use_drop_idx: 64,
                    masking_drop_idx: 64,
                    drop_counter: 0,
                    sch_counter: 0,
                    masking_sch_idx: 64,
                    use_sch_idx: 64,
                    order: MAX_RING_BUFFER,
                    job_order: MAX_RING_BUFFER,
                };

                while !block_clone.load(Ordering::Acquire) {
                    spin_loop();
                }

                // running
                thread_unit.running();
            })
        });

        block.store(true, Ordering::Release);

        Self {
            done_task,
            join_flag,
            packet_core: list_core,
            pool,
        }
    }

    pub fn join(self) {
        unsafe {
            // clean
            // check, all task done
            loop {
                if self.packet_core.in_task.load(Ordering::Relaxed)
                    <= self.done_task.load(Ordering::Relaxed)
                {
                    break;
                }
                spin_loop();
            }

            // join
            self.join_flag.store(true, Ordering::Release);
            for join_handle in self.pool {
                join_handle.join().unwrap();
            }

            // clean quota
            let quota_idx = self.packet_core.use_quota.load(Ordering::Relaxed);
            let mut quota_list = Box::from_raw(
                self.packet_core
                    .quota_list
                    .swap(null_mut(), Ordering::Relaxed),
            );

            if quota_idx < 64 {
                println!("drop dilakukan oleh cahotic");
                quota_list[quota_idx].free();
            }
            drop(quota_list);

            // clean schedule_list
            drop(Box::from_raw(
                self.packet_core
                    .schedule_list
                    .swap(null_mut(), Ordering::Relaxed),
            ));
        }
    }
}
