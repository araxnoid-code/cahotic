use std::sync::Arc;

use crate::{
    ListCore, OutputTrait, PoolTask, TaskTrait, TaskWithDependenciesTrait, ThreadPoolCore,
};

pub struct Cahotic<F, FD, O, const N: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send + Send,
{
    // List Core
    list_core: Arc<ListCore<F, FD, O>>,
    // thread pool Core
    thread_pool_core: ThreadPoolCore<F, FD, O, N>,
}

impl<F, FD, O, const N: usize> Cahotic<F, FD, O, N>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> Cahotic<F, FD, O, N> {
        let list_core = Arc::new(ListCore::<F, FD, O>::init());
        let thread_pool_core = ThreadPoolCore::<F, FD, O, N>::init(list_core.clone());
        Self {
            list_core,
            thread_pool_core,
        }
    }

    pub fn spawn_task(&self, f: F) -> PoolTask<O> {
        self.list_core.spawn_task(f)
    }

    pub fn join(self) {
        self.thread_pool_core.join();
    }
}
