use crate::{OutputTrait, SchedulerTrait, TaskTrait, WaitingTask};

// Dequeue
pub enum DequeueStatus<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Ok(WaitingTask<F, FS, O>),
    Waiting(usize),
    None,
}

// Try Enqueue
#[derive(Debug)]
pub enum TryEnqueueStatus {
    QuotaFull,
    RingBufferFull,
}
