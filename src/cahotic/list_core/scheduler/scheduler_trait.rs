use crate::{OutputTrait, SchedulerVec};

pub trait SchedulerTrait<O>
where
    O: OutputTrait + 'static + Send,
{
    fn execute(&self, dependencies: SchedulerVec<O>) -> O;
}
