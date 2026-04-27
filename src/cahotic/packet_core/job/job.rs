use std::{
    cell::RefCell,
    ops::Deref,
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering},
    },
};

use crate::{OutputTrait, TaskTrait, WaitingTask};

/// JobUnit
#[repr(align(64))]
pub(crate) struct JobUnit<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: JobTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) inner: Option<WaitingTask<F, FS, O>>,
    pub(crate) empty: AtomicBool,
}

impl<F, FS, O> JobUnit<F, FS, O>
where
    F: TaskTrait<O> + Send + 'static,
    FS: JobTrait<O> + Send + 'static,
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
    FS: JobTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) inner: Arc<InnerJob<FS, O>>,
}

impl<FS, O> Job<FS, O>
where
    FS: JobTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn create_job(task: FS) -> Job<FS, O> {
        Self {
            inner: Arc::new(InnerJob::create_job(task)),
        }
    }

    pub fn after(&self, job: &Job<FS, O>) {
        self.inner.exec_counter.fetch_add(1, Ordering::Relaxed);
        self.inner
            .return_ptr_list
            .borrow_mut()
            .push(job.inner.return_ptr);
        self.inner
            .parent_quota
            .borrow_mut()
            .push(AtomicUsize::new(64));

        job.inner.child_counter.fetch_add(1, Ordering::Relaxed);
        job.inner.child_list.borrow_mut().push(self.clone_inner());
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
    FS: JobTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) task: FS,
    pub(crate) child_counter: AtomicUsize,
    pub(crate) exec_counter: AtomicUsize,
    pub(crate) child_list: RefCell<Vec<Arc<InnerJob<FS, O>>>>,
    pub(crate) return_ptr: &'static AtomicPtr<O>,
    pub(crate) return_ptr_list: RefCell<Vec<&'static AtomicPtr<O>>>,
    pub(crate) parent_quota_head: AtomicUsize,
    pub(crate) parent_quota: RefCell<Vec<AtomicUsize>>,
}

impl<FS, O> InnerJob<FS, O>
where
    FS: JobTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn create_job(task: FS) -> InnerJob<FS, O> {
        Self {
            task,
            child_counter: AtomicUsize::new(0),
            exec_counter: AtomicUsize::new(0),
            child_list: RefCell::new(vec![]),
            return_ptr: Box::leak(Box::new(AtomicPtr::new(null_mut()))),
            return_ptr_list: RefCell::new(vec![]),
            parent_quota_head: AtomicUsize::new(0),
            parent_quota: RefCell::new(vec![]),
        }
    }

    pub fn push_parent_quota(&self, quota_idx: usize) {
        let idx = self.parent_quota_head.fetch_add(1, Ordering::Relaxed);
        self.parent_quota.borrow()[idx].store(quota_idx, Ordering::Relaxed);
    }
}

///
pub struct JobVec<O>
where
    O: 'static + OutputTrait + Send,
{
    pub(crate) vec: Vec<&'static AtomicPtr<O>>,
}

impl<O> JobVec<O>
where
    O: 'static + OutputTrait + Send,
{
    pub fn get(&self, idx: usize) -> Option<&O> {
        unsafe {
            if let Some(ptr) = self.vec.get(idx) {
                Some(&*ptr.load(Ordering::Acquire))
            } else {
                None
            }
        }
    }
}

pub trait JobTrait<O>
where
    O: OutputTrait + 'static + Send,
{
    fn execute(&self, scheduler_vec: JobVec<O>) -> O;
}
