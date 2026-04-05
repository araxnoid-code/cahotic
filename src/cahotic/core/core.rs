use std::sync::Arc;

use crate::{
    OutputTrait, PollWaiting, Schedule, SchedulerTrait, TaskCore, TaskTrait, ThreadPoolCore,
};

pub struct Cahotic<F, FS, O, const N: usize, const PN: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    // task Core
    pub task_core: Arc<TaskCore<F, FS, O, PN>>,

    // thread pool Core
    thread_pool_core: ThreadPoolCore<F, FS, O, N, PN>,
}

impl<F, FS, O, const N: usize, const PN: usize> Cahotic<F, FS, O, N, PN>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
    pub fn init() -> Cahotic<F, FS, O, N, PN> {
        let list_core = Arc::new(TaskCore::<F, FS, O, PN>::init());
        let thread_pool_core = ThreadPoolCore::<F, FS, O, N, PN>::init(list_core.clone());
        Self {
            task_core: list_core,
            thread_pool_core,
        }
    }

    // spawn task
    pub fn spawn_task(&self, f: F) -> PollWaiting<O> {
        self.task_core.spawn_task(f)
    }

    // packet
    pub fn submit_packet(&self) {
        self.task_core.submit_packet();
    }

    // scheduling
    pub fn schedule_exec(&self, schedule: Schedule<F, FS, O>) -> PollWaiting<O> {
        self.task_core.schedule_exec(schedule)
    }

    pub fn scheduling_create_initial(&self, task: F) -> Schedule<F, FS, O> {
        self.task_core.packet_core.scheduling_create_initial(task)
    }

    pub fn scheduling_create_schedule(&self, schedule: FS) -> Schedule<F, FS, O> {
        self.task_core
            .packet_core
            .scheduling_create_schedule(schedule)
    }

    pub fn schedule_after(
        &self,
        schedule: &mut Schedule<F, FS, O>,
        after: &mut Schedule<F, FS, O>,
    ) -> Result<(), &str> {
        self.task_core.packet_core.schedule_after(schedule, after)
    }

    // end
    pub fn join(self) {
        self.thread_pool_core.join();
    }
}
