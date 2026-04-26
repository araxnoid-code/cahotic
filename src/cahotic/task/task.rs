use std::sync::{
    Arc,
    atomic::{AtomicPtr, AtomicUsize},
};

use crate::{InnerJob, OutputTrait, SchedulerTrait, TaskTrait};

/// ExecTask
pub(crate) enum ExecTask<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F),
    Scheduling(
        FS,
        Vec<&'static AtomicPtr<O>>,
        usize,
        Vec<&'static AtomicUsize>,
    ),
    Job(Arc<InnerJob<FS, O>>),
}
