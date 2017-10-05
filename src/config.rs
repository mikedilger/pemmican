
use std::time::Duration;

/// Configuration settings for a Pemmican server instance
pub struct Config {
    /// Number of threads for the CpuPool.  Note that handler functions are run on the
    /// main thread, and you must use `pemmican.pool` if you want to run code on a
    /// separate thread in the Pemmican CpuPool.  Defaults to 4.
    pub num_threads: usize,

    /// Configure the amount of time the server will wait for a "graceful shutdown".
    /// This is the amount of time after the shutdown signal is received the server
    /// will wait for all pending connections to finish. If the timeout elapses then
    /// the server will be forcibly shut down.  Defaults to 1s.
    pub shutdown_timeout: Duration,

    /// Enable or disable Keep-alive.  Default is true.
    pub keep_alive: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            num_threads: 4,
            shutdown_timeout: Duration::from_secs(1),
            keep_alive: true,
        }
    }
}
