use std::{
    hint::spin_loop,
    sync::atomic::{AtomicPtr, Ordering},
};

pub struct PoolWait<O>
where
    O: 'static,
{
    pub(crate) data_ptr: &'static AtomicPtr<O>,
}

impl<O> PoolWait<O> {
    pub fn block(&self) -> &O {
        while self.data_ptr.load(Ordering::Acquire).is_null() {
            spin_loop();
        }

        unsafe { &*self.data_ptr.load(Ordering::Acquire) }
    }

    pub fn get(&self) -> Option<&O> {
        let data = self.data_ptr.load(Ordering::Acquire);
        if !data.is_null() {
            unsafe { Some(&*self.data_ptr.load(Ordering::Acquire)) }
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
