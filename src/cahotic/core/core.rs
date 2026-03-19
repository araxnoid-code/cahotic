use std::{fmt::Debug, sync::Arc};

use crate::{
    ListCore, OutputTrait, PollWaiting, Scheduler, SchedulerTrait, TaskTrait, ThreadPoolCore,
};

pub struct Cahotic<F, FD, O, const N: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FD: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send + Send,
{
    // List Core
    list_core: Arc<ListCore<F, FD, O>>,
    // thread pool Core
    thread_pool_core: ThreadPoolCore<F, FD, O, N>,
}

impl<F, FS, O, const N: usize> Cahotic<F, FS, O, N>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
    pub fn init() -> Cahotic<F, FS, O, N> {
        let list_core = Arc::new(ListCore::<F, FS, O>::init());
        let thread_pool_core = ThreadPoolCore::<F, FS, O, N>::init(list_core.clone());
        Self {
            list_core,
            thread_pool_core,
        }
    }

    pub fn spawn_task(&self, f: F) -> PollWaiting<O> {
        self.list_core.spawn_task(f)
    }

    pub fn drop_poll(&self, pool_waiting: PollWaiting<O>) {
        self.list_core.drop_pool(pool_waiting);
    }

    pub fn swap_drop_arena(&self) {
        self.list_core.swap_drop_arena();
    }

    pub fn scheduler_exec(&self, scheduler: Scheduler<FS, O>) -> PollWaiting<O> {
        self.list_core.scheduler_exec(scheduler)
    }

    // pub fn spwan_dependencies<D>(&self, dependencies: D) -> TaskDependencies<F, FD, O>
    // where
    //     D: TaskDependenciesTrait<F, O>,
    // {
    //     self.list_core.spawn_dependencies(dependencies)
    // }

    // pub fn spawn_task_with_dependencies(
    //     &self,
    //     task: FD,
    //     dependencies: &TaskDependencies<F, FD, O>,
    // ) -> PollWaiting<O> {
    //     self.list_core
    //         .spawn_task_with_dependencies(task, dependencies, None)
    // }

    pub fn join(self) {
        self.thread_pool_core.join();
    }
}
