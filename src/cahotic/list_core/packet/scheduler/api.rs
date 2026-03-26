use std::sync::atomic::Ordering;

use crate::{OutputTrait, PollWaiting, Schedule, SchedulerTrait, TaskCore, TaskTrait};

impl<F, FS, O, const PN: usize> TaskCore<F, FS, O, PN>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn schedule_exec(&self, schedule: Schedule<F, FS, O>) -> PollWaiting<O> {
        self.packet_core.add_schedule(
            schedule,
            self.id_counter.fetch_add(1, Ordering::Release),
            &self.in_task,
        )
    }
}
