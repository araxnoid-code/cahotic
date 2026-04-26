use std::{
    cell::RefCell,
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicPtr, AtomicUsize, Ordering},
    },
};

use crate::{OutputTrait, SchedulerTrait};

/// Job
pub struct Job<FS, O>
where
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    inner: Arc<InnerJob<FS, O>>,
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

    //
    pub fn inner(&self) -> &Arc<InnerJob<FS, O>> {
        &self.inner
    }

    pub fn clone_inner(&self) -> Arc<InnerJob<FS, O>> {
        self.inner.clone()
    }
}

/// InnerJob
struct InnerJob<FS, O>
where
    FS: SchedulerTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    task: FS,
    counter: AtomicUsize,
    job_list: RefCell<Vec<Arc<InnerJob<FS, O>>>>,
    return_ptr: &'static AtomicPtr<O>,
    return_ptr_list: RefCell<Vec<&'static AtomicPtr<O>>>,
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
