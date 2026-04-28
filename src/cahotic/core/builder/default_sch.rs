use crate::{JobTrait, JobVec, OutputTrait};

pub struct DefaultJob<O>(pub fn(vector: JobVec<O>) -> O)
where
    O: OutputTrait + 'static + Send;

impl<O> JobTrait<O> for DefaultJob<O>
where
    O: OutputTrait + 'static + Send,
{
    fn execute(&self, scheduler_vec: JobVec<O>) -> O {
        (self.0)(scheduler_vec)
    }
}
