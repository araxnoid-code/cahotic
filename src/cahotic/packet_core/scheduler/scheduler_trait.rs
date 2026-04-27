use std::sync::atomic::{AtomicPtr, Ordering};

use crate::OutputTrait;

// pub struct ScheduleVec<O>
// where
//     O: 'static + OutputTrait + Send,
// {
//     pub(crate) vec: Vec<&'static AtomicPtr<O>>,
// }

// impl<O> ScheduleVec<O>
// where
//     O: 'static + OutputTrait + Send,
// {
//     pub fn get(&self, idx: usize) -> Option<&O> {
//         unsafe {
//             if let Some(ptr) = self.vec.get(idx) {
//                 Some(&*ptr.load(Ordering::Acquire))
//             } else {
//                 None
//             }
//         }
//     }
// }

// pub trait SchedulerTrait<O>
// where
//     O: OutputTrait + 'static + Send,
// {
//     fn execute(&self, scheduler_vec: ScheduleVec<O>) -> O;
// }
