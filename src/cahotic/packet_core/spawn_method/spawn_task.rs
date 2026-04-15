use crate::{OutputTrait, PacketCore, PollWaiting, SchedulerTrait, TaskTrait};

impl<F, FS, O, const MAX_RING_BUFFER: usize> PacketCore<F, FS, O, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn spawn_task(&self, task: F) -> PollWaiting<O> {
        self.enqueue(task)
    }

    pub fn try_spawn_task(&self, task: F) -> Result<PollWaiting<O>, crate::TryEnqueueStatus> {
        self.try_enqueue(task)
    }
}
