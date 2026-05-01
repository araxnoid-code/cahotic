use crate::{DependenciesVec, JobTrait, OutputTrait};

pub struct DefaultJob<O>(pub fn(vector: DependenciesVec<O>) -> O)
where
    O: OutputTrait + 'static + Send;

impl<O> JobTrait<O> for DefaultJob<O>
where
    O: OutputTrait + 'static + Send,
{
    fn execute(&self, scheduler_vec: DependenciesVec<O>) -> O {
        (self.0)(scheduler_vec)
    }
}
