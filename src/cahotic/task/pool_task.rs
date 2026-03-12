use std::sync::atomic::AtomicPtr;

pub struct PoolTask<O>
where
    O: 'static,
{
    pub(crate) data_ptr: &'static AtomicPtr<O>,
}
