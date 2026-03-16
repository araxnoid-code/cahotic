use crate::{
    ExecTask, OutputTrait, PollWaiting, TaskDependencies, TaskTrait, TaskWithDependenciesTrait,
};

pub trait DropAfterTrait<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    fn get_exec_task(self, poll_wait: &PollWaiting<O>) -> ExecTask<F, FD, O>;
}

impl<F, FD, O> DropAfterTrait<F, FD, O> for PollWaiting<O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    fn get_exec_task(self, poll_wait: &PollWaiting<O>) -> ExecTask<F, FD, O> {
        ExecTask::DropPollAfter(self, (poll_wait.data_ptr, poll_wait.drop_after_caounter))
    }
}

impl<F, FD, O> DropAfterTrait<F, FD, O> for TaskDependencies<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    fn get_exec_task(self, poll_wait: &PollWaiting<O>) -> ExecTask<F, FD, O> {
        ExecTask::DropDependenciesAfter(self, (poll_wait.data_ptr, poll_wait.drop_after_caounter))
    }
}
