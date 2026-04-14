use crate::{OutputTrait, TaskTrait};

pub struct DefaultTask<O>(pub fn() -> O)
where
    O: OutputTrait + 'static;

impl<O> TaskTrait<O> for DefaultTask<O>
where
    O: OutputTrait + 'static,
{
    fn execute(&self) -> O {
        (self.0)()
    }
}
