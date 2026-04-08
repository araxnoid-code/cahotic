use crate::{OutputTrait, PacketCore, PollWaiting, Schedule, SchedulerTrait, TaskTrait};

impl<F, FS, O> PacketCore<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn schedule_exec(&self, schedule: Schedule<F, FS, O>) -> PollWaiting<O> {
        self.schedule_enqueue(schedule)
    }
}
