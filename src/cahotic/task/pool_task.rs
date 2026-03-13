use std::{
    hint::spin_loop,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{OutputTrait, TaskDependenciesCore, TaskTrait, TaskWithDependenciesTrait};

pub struct PoolWait<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    output: PoolOutput<O>,
    dependencies: &'static TaskDependenciesCore<F, FD, O>,
}

impl<F, FD, O> PoolWait<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn block(&self) -> &O {
        self.output.block()
    }

    pub fn get(&self) -> Option<&O> {
        self.output.get()
    }

    pub fn collect(self) -> O {
        self.output.collect()
    }
}

pub struct PoolOutput<O>
where
    O: 'static,
{
    pub(crate) data_ptr: &'static AtomicPtr<O>,
}

impl<O> PoolOutput<O> {
    pub fn block(&self) -> &O {
        while self.data_ptr.load(Ordering::Acquire).is_null() {
            spin_loop();
        }

        unsafe { &*self.data_ptr.load(Ordering::Acquire) }
    }

    pub fn get(&self) -> Option<&O> {
        let data = self.data_ptr.load(Ordering::Acquire);
        if !data.is_null() {
            unsafe { Some(&*data) }
        } else {
            None
        }
    }

    pub fn collect(self) -> O {
        unsafe {
            while self.data_ptr.load(Ordering::Acquire).is_null() {
                spin_loop();
            }

            let data_box = Box::from_raw(self.data_ptr.load(Ordering::Acquire));
            *data_box
        }
    }
}
