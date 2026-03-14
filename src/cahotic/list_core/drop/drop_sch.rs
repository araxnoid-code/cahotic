use std::fmt::Debug;

use crate::{OutputTrait, PoolWait, TaskTrait, TaskWithDependenciesTrait};

pub struct DropSchedule<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub pool_wait: PoolWait<F, FD, O>,
}
