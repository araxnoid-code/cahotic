use std::fmt::Debug;

use crate::{DropSchedule, OutputTrait, TaskTrait, TaskWithDependenciesTrait};

pub enum ExecTask<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F),
    TaskWithDependencies(FD),
    Drop(DropSchedule<F, FD, O>),
    Output(O),
}
