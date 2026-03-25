use std::sync::atomic::Ordering;

use crate::{ListCore, OutputTrait, PollWaiting, SchedulerTrait, TaskTrait};

impl<F, FS, O, const PN: usize> ListCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn spawn_task(&self, task: F) -> PollWaiting<O> {
        self.packet_core.add_task(
            task,
            self.id_counter.fetch_add(1, Ordering::Release),
            &self.in_task,
        )
    }
}
