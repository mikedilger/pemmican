
use futures_cpupool::CpuPool;

/// A Shared component within Pemmican, accessible to plugins
pub struct Shared<S>
{
    pub pool: CpuPool,
    #[allow(dead_code)] // this is provided for handlers; this library does not use it
    pub state: S,
}

impl<S> Shared<S>
{
    pub fn new(num_threads: usize, state: S) -> Shared<S> {
        Shared {
            pool: CpuPool::new(num_threads),
            state: state,
        }
    }
}
