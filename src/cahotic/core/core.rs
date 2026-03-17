use std::{fmt::Debug, sync::Arc};

use crate::{
    DropAfterTrait, ListCore, OutputTrait, PollWaiting, TaskDependencies, TaskDependenciesTrait,
    TaskTrait, TaskWithDependenciesTrait, ThreadPoolCore,
};

pub struct Cahotic<F, FD, O, const N: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send + Send + Debug,
{
    // List Core
    list_core: Arc<ListCore<F, FD, O>>,
    // thread pool Core
    thread_pool_core: ThreadPoolCore<F, FD, O, N>,
}

impl<F, FD, O, const N: usize> Cahotic<F, FD, O, N>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FD: TaskWithDependenciesTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Debug + Sync,
{
    pub fn init() -> Cahotic<F, FD, O, N> {
        let list_core = Arc::new(ListCore::<F, FD, O>::init());
        let thread_pool_core = ThreadPoolCore::<F, FD, O, N>::init(list_core.clone());
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

    pub fn drop_after<D>(&self, drop: D, poll_waiting: &PollWaiting<O>)
    where
        D: DropAfterTrait<F, FD, O>,
    {
        self.list_core.drop_after(drop, poll_waiting);
    }

    pub fn swap_drop_arena(&self) {
        self.list_core.swap_drop_arena();
    }

    pub fn drop_dependencies(&self, dependencies: TaskDependencies<F, FD, O>) {
        self.list_core.drop_dependencies(dependencies);
    }

    pub fn spwan_dependencies<D>(&self, dependencies: D) -> TaskDependencies<F, FD, O>
    where
        D: TaskDependenciesTrait<F, O>,
    {
        self.list_core.spawn_dependencies(dependencies)
    }

    pub fn spawn_task_with_dependencies(
        &self,
        task: FD,
        dependencies: &TaskDependencies<F, FD, O>,
    ) -> PollWaiting<O> {
        self.list_core
            .spawn_task_with_dependencies(task, dependencies, None)
    }

    pub fn join(self) {
        self.thread_pool_core.join();
    }
}
