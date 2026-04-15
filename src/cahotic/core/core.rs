use std::sync::Arc;

use crate::{
    OutputTrait, PacketCore, PollWaiting, Schedule, SchedulerTrait, TaskTrait, ThreadPoolCore,
};

pub struct Cahotic<F, FS, O, const N: usize, const MAX_RING_BUFFER: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // task Core
    pub(crate) packet_core: Arc<PacketCore<F, FS, O, MAX_RING_BUFFER>>,

    // thread pool Core
    pub(crate) thread_pool_core: ThreadPoolCore<F, FS, O, N, MAX_RING_BUFFER>,
}

impl<F, FS, O, const N: usize, const MAX_RING_BUFFER: usize> Cahotic<F, FS, O, N, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
    pub fn init() -> Result<Cahotic<F, FS, O, N, MAX_RING_BUFFER>, &'static str> {
        if MAX_RING_BUFFER & 63 != 0 || MAX_RING_BUFFER <= 0 {
            return Err(
                "build error, The size for the ring buffer must be greater than 0 and must be a multiple of 64.",
            );
        }

        let list_core = Arc::new(PacketCore::<F, FS, O, MAX_RING_BUFFER>::init());
        let thread_pool_core =
            ThreadPoolCore::<F, FS, O, N, MAX_RING_BUFFER>::init(list_core.clone());

        Ok(Self {
            packet_core: list_core,
            thread_pool_core,
        })
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
