use std::marker::PhantomData;

use crate::{
    Cahotic, DefaultOutput, DefaultSchedule, DefaultTask, OutputTrait, SchedulerTrait, TaskTrait,
};

pub struct CahoticBuilder<F, FS, O, const N: usize, const MAX_RING_BUFFER: usize>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
    task: PhantomData<F>,
    schedule: PhantomData<FS>,
    output: PhantomData<O>,
}

impl
    CahoticBuilder<
        DefaultTask<DefaultOutput<usize>>,
        DefaultSchedule<DefaultOutput<usize>>,
        DefaultOutput<usize>,
        4,
        4096,
    >
{
    pub fn default<OutputType>() -> CahoticBuilder<
        DefaultTask<DefaultOutput<OutputType>>,
        DefaultSchedule<DefaultOutput<OutputType>>,
        DefaultOutput<OutputType>,
        4,
        4096,
    >
    where
        OutputType: 'static + Send + Sync,
    {
        CahoticBuilder {
            output: PhantomData::default(),
            schedule: PhantomData::default(),
            task: PhantomData::default(),
        }
    }
}

impl<F, FS, O, const N: usize, const MAX_RING_BUFFER: usize>
    CahoticBuilder<F, FS, O, N, MAX_RING_BUFFER>
where
    F: TaskTrait<O> + 'static + Send + Sync,
    FS: SchedulerTrait<O> + Send + 'static + Sync,
    O: 'static + OutputTrait + Send + Sync,
{
    pub fn set_task_type<T>(&self) -> CahoticBuilder<T, FS, O, N, MAX_RING_BUFFER>
    where
        T: TaskTrait<O> + 'static + Send + Sync,
    {
        CahoticBuilder {
            output: PhantomData::default(),
            schedule: PhantomData::default(),
            task: PhantomData::default(),
        }
    }

    pub fn set_schedule_type<T>(&self) -> CahoticBuilder<F, T, O, N, MAX_RING_BUFFER>
    where
        T: SchedulerTrait<O> + Send + 'static + Sync,
    {
        CahoticBuilder {
            output: PhantomData::default(),
            schedule: PhantomData::default(),
            task: PhantomData::default(),
        }
    }

    pub fn set_type<TASK, SCHEDULE, OUTPUT>(
        &self,
    ) -> CahoticBuilder<TASK, SCHEDULE, OUTPUT, N, MAX_RING_BUFFER>
    where
        TASK: TaskTrait<OUTPUT> + 'static + Send + Sync,
        SCHEDULE: SchedulerTrait<OUTPUT> + Send + 'static + Sync,
        OUTPUT: 'static + OutputTrait + Send + Sync,
    {
        CahoticBuilder {
            output: PhantomData::default(),
            schedule: PhantomData::default(),
            task: PhantomData::default(),
        }
    }

    pub fn set_workers<const W: usize>(&self) -> CahoticBuilder<F, FS, O, W, MAX_RING_BUFFER> {
        CahoticBuilder {
            output: PhantomData::default(),
            schedule: PhantomData::default(),
            task: PhantomData::default(),
        }
    }

    pub fn set_ring_buffer_size<const MAX: usize>(&self) -> CahoticBuilder<F, FS, O, N, MAX> {
        CahoticBuilder {
            output: PhantomData::default(),
            schedule: PhantomData::default(),
            task: PhantomData::default(),
        }
    }

    pub fn build(&self) -> Result<Cahotic<F, FS, O, N, MAX_RING_BUFFER>, &'static str> {
        if MAX_RING_BUFFER & 63 != 0 || MAX_RING_BUFFER <= 0 {
            return Err(
                "build error, The size for the ring buffer must be greater than 0 and must be a multiple of 64.",
            );
        }

        Cahotic::init()
    }
}
