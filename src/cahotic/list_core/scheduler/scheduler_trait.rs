use std::sync::Arc;

use crate::{ListCore, OutputTrait, ScheduleVec};

pub trait SchedulerTrait<O>
where
    O: OutputTrait + 'static + Send,
{
    fn execute(&self, scheduler_vec: ScheduleVec<O>) -> O;
}

// struct Scheduler {}
