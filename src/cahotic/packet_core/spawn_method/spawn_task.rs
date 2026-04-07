use crate::{OutputTrait, PacketCore, PollWaiting, SchedulerTrait, TaskTrait};

impl<F, FS, O> PacketCore<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn spawn_task(&self, task: F) -> PollWaiting<O> {
        self.enqueue(task)
    }
}
