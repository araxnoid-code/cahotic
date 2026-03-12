use crate::{OutputTrait, TaskTrait, TaskWithDependenciesTrait};

pub enum ExecTask<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F),
    TaskWithDependencies(FD),
    _Output(O),
}
