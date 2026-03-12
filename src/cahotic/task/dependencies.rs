use crate::{OutputTrait, PoolTask};

pub trait TaskWithDependenciesTrait<O>
where
    O: OutputTrait + 'static + Send,
{
    fn exec(&self, dependencies: &'static Vec<PoolTask<O>>) -> O;

    fn is_with_dependencies(&self) -> bool {
        true
    }
}
