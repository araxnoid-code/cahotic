use crate::OutputTrait;

pub struct DefaultOutput<O>(pub O)
where
    O: 'static;

impl<O> OutputTrait for DefaultOutput<O> where O: 'static {}
