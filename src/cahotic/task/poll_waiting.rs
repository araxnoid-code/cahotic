use std::{
    hint::spin_loop,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

pub struct PollWaiting<O>
where
    O: 'static,
{
    pub(crate) data_ptr: &'static AtomicPtr<O>,
    pub(crate) drop_after_caounter: &'static AtomicUsize,
}

impl<O> PollWaiting<O> {
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

    pub fn collect(&self) -> &O {
        unsafe {
            while self.data_ptr.load(Ordering::Acquire).is_null() {
                spin_loop();
            }

            let data = self.data_ptr.load(Ordering::Acquire);
            &*data
        }
    }
}
