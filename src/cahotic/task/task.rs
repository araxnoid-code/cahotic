use std::sync::Arc;

use crate::{InnerJob, JobTrait, OutputTrait, TaskTrait};

/// ExecTask
pub(crate) enum ExecTask<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: JobTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F),
    Job(Arc<InnerJob<FS, O>>),
}
