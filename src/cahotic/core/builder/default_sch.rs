use crate::{OutputTrait, ScheduleVec, SchedulerTrait};

pub struct DefaultSchedule<O>(pub fn(vector: ScheduleVec<O>) -> O)
where
    O: OutputTrait + 'static + Send;

impl<O> SchedulerTrait<O> for DefaultSchedule<O>
where
    O: OutputTrait + 'static + Send,
{
    fn execute(&self, scheduler_vec: ScheduleVec<O>) -> O {
        (self.0)(scheduler_vec)
    }
}
