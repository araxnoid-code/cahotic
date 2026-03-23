use std::{
    fmt::Debug,
    hint::spin_loop,
    sync::{Arc, atomic::Ordering},
    thread::sleep,
    time::Duration,
};

use crate::{
    ListCore, OutputTrait, PollWaiting, Scheduler, SchedulerTrait, TaskTrait, ThreadPoolCore,
};

pub struct Cahotic<F, FD, O, const N: usize, const PN: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send + Send,
{
    // List Core
    pub list_core: Arc<ListCore<F, FD, O, PN>>,
    limit: u64,
    // thread pool Core
    thread_pool_core: ThreadPoolCore<F, FD, O, N, PN>,
}

impl<F, FS, O, const N: usize, const PN: usize> Cahotic<F, FS, O, N, PN>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
    pub fn init() -> Cahotic<F, FS, O, N, PN> {
        let list_core = Arc::new(ListCore::<F, FS, O, PN>::init());
        let thread_pool_core = ThreadPoolCore::<F, FS, O, N, PN>::init(list_core.clone());
        Self {
            list_core,
            limit: 10_000,
            thread_pool_core,
        }
    }

    pub fn spawn_task(&self, f: F) -> PollWaiting<O> {
        self.list_core.spawn_task(f)
    }

    pub fn submit_packet(&self) {
        self.list_core.submit_packet();
    }

    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    pub fn drop_poll(&self, pool_waiting: PollWaiting<O>) {
        // self.list_core.drop_pool(pool_waiting);
    }

    pub fn drop_arena(&self) {
        // self.list_core.drop_arena();

        // println!(
        //     "{}/{}",
        //     self.list_core.in_task.load(Ordering::SeqCst),
        //     self.thread_pool_core.done_task.load(Ordering::SeqCst)
        // );
        // sleep(Duration::from_millis(100));
    }

    pub fn scheduler_exec(&self, scheduler: Scheduler<FS, O>) -> PollWaiting<O> {
        self.list_core.scheduler_exec(scheduler)
    }

    pub fn join(self) {
        self.thread_pool_core.join();
    }
}
