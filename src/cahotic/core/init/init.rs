use crate::{OutputTrait, SchedulerTrait, TaskTrait};

pub struct CahoticInit<F, FS, O, const N: usize, const MAX_RING_BUFFER: usize>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
    task: F,
    schedule: FS,
    output: O,
    total_worker_thread: usize,
    ring_buffer_size: usize,
}

impl<F, FS, O, const N: usize, const MAX_RING_BUFFER: usize>
    CahoticInit<F, FS, O, N, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
}
