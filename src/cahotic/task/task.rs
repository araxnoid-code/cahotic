use std::fmt::Debug;

use crate::{DropSchedule, OutputTrait, TaskDependencies, TaskTrait, TaskWithDependenciesTrait};

pub enum ExecTask<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F),
    TaskWithDependencies(FD),
    DropPool(DropSchedule<F, FD, O>),
    DropDependencies(TaskDependencies<F, FD, O>),
    Output(O),
    None,
}
