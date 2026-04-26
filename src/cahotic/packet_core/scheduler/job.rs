use std::{
    cell::RefCell,
    ops::Deref,
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering},
    },
};

use crate::{OutputTrait, SchedulerTrait, TaskTrait, WaitingTask};

/// JobUnit
#[repr(align(64))]
pub(crate) struct JobUnit<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) inner: Option<WaitingTask<F, FS, O>>,
    pub(crate) empty: AtomicBool,
}

impl<F, FS, O> JobUnit<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> JobUnit<F, FS, O> {
        Self {
            inner: None,
            empty: AtomicBool::new(true),
        }
    }
}

/// JobCounter
#[repr(align(64))]
pub(crate) struct JobCounter {
    head: AtomicU64,
}

impl Default for JobCounter {
    fn default() -> Self {
        Self {
            head: AtomicU64::new(0),
        }
    }
}

impl Deref for JobCounter {
    type Target = AtomicU64;
    fn deref(&self) -> &Self::Target {
        &self.head
    }
}

/// Job
pub struct Job<FS, O>
where
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) inner: Arc<InnerJob<FS, O>>,
}

impl<FS, O> Job<FS, O>
where
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn create_job(task: FS) -> Job<FS, O> {
        Self {
            inner: Arc::new(InnerJob::create_job(task)),
        }
    }

    pub fn after(&self, job: &Job<FS, O>) {
        self.inner.counter.fetch_add(1, Ordering::Relaxed);
        self.inner
            .return_ptr_list
            .borrow_mut()
            .push(job.inner.return_ptr);

        job.inner.job_list.borrow_mut().push(self.clone_inner());
    }

    // additional methods
    pub fn inner(&self) -> &Arc<InnerJob<FS, O>> {
        &self.inner
    }

    pub fn clone_inner(&self) -> Arc<InnerJob<FS, O>> {
        self.inner.clone()
    }
}

/// InnerJob
pub struct InnerJob<FS, O>
where
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) task: FS,
    pub(crate) counter: AtomicUsize,
    pub(crate) job_list: RefCell<Vec<Arc<InnerJob<FS, O>>>>,
    pub(crate) return_ptr: &'static AtomicPtr<O>,
    pub(crate) return_ptr_list: RefCell<Vec<&'static AtomicPtr<O>>>,
}

impl<FS, O> InnerJob<FS, O>
where
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn create_job(task: FS) -> InnerJob<FS, O> {
        Self {
            task,
            counter: AtomicUsize::new(0),
            job_list: RefCell::new(vec![]),
            return_ptr: Box::leak(Box::new(AtomicPtr::new(null_mut()))),
            return_ptr_list: RefCell::new(vec![]),
        }
    }
}
