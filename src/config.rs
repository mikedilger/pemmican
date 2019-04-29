
use std::time::Duration;

/// Configuration settings for a Pemmican server instance
pub struct Config {
    /// Number of threads for the CpuPool.  Note that handler functions are run on the
    /// main thread, and you must use `pemmican.pool` if you want to run code on a
    /// separate thread in the Pemmican CpuPool.  Defaults to 4.
    pub num_threads: usize,

    /// Enable or disable Keep-alive.  Default is true.
    pub keep_alive: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            num_threads: 4,
            keep_alive: true,
        }
    }
}
