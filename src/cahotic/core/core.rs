use std::sync::Arc;

use crate::{
    OutputTrait, PacketCore, PollWaiting, Schedule, SchedulerTrait, TaskTrait, ThreadPoolCore,
};

pub struct Cahotic<F, FS, O, const N: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // task Core
    pub(crate) packet_core: Arc<PacketCore<F, FS, O>>,

    // thread pool Core
    pub(crate) thread_pool_core: ThreadPoolCore<F, FS, O, N>,
}

impl<F, FS, O, const N: usize> Cahotic<F, FS, O, N>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
    pub fn init() -> Cahotic<F, FS, O, N> {
        let list_core = Arc::new(PacketCore::<F, FS, O>::init());
        let thread_pool_core = ThreadPoolCore::<F, FS, O, N>::init(list_core.clone());
        Self {
            packet_core: list_core,
            thread_pool_core,
        }
    }

    // spawn task
    pub fn spawn_task(&self, f: F) -> PollWaiting<O> {
        self.packet_core.spawn_task(f)
    }

    // scheduling
    pub fn schedule_exec(&self, schedule: Schedule<F, FS, O>) -> PollWaiting<O> {
        self.packet_core.schedule_exec(schedule)
    }

    pub fn scheduling_create_initial(&self, task: F) -> Schedule<F, FS, O> {
        self.packet_core.scheduling_create_initial(task)
    }

    pub fn scheduling_create_schedule(&self, schedule: FS) -> Schedule<F, FS, O> {
        self.packet_core.scheduling_create_schedule(schedule)
    }

    pub fn schedule_after(
        &self,
        schedule: &mut Schedule<F, FS, O>,
        after: &mut Schedule<F, FS, O>,
    ) -> Result<(), &str> {
        self.packet_core.schedule_after(schedule, after)
    }

    // end
    pub fn join(self) {
        self.thread_pool_core.join();
    }
}
