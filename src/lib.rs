
extern crate hyper;
extern crate futures;
extern crate futures_cpupool;
extern crate chashmap;

pub mod error;
pub use error::Error;

use chashmap::CHashMap;
use futures::Future;
use futures::future;
use futures_cpupool::CpuPool;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};
use std::time::Duration;
use std::sync::Arc;
use std::error::Error as StdError;

/// Handler functions handle web requests by generating a response. Code within
/// these functions should take care not to block or call functions which may block.
/// Instead, they should return futures immediately.
///
/// In the case of an error, if at all possible, respond to the client with a 5xx
/// error code and return a Response rather than an Error.
///
/// A reference to pemmican will be supplied to the request, so that you have access
/// to the services it provides (such as access to your state <S> and access to the
/// thread pool).
pub type Handler<S, E> =
    fn(pemmican: &Pemmican<S, E>, Request)
       -> Box<Future<Item = Response, Error = E>>;

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

/// A Pemmican server instance.
pub struct Pemmican<S: Send + Sync + 'static,
                    E: StdError + Send + Sync + 'static>
{
    routes: CHashMap<(String, Method), Handler<S, E>>,
    pub pool: CpuPool,
    config: Config,
    #[allow(dead_code)] // this is provided for handlers; this library does not use it
    pub state: S,
}

impl<S: Send + Sync + 'static,
     E: StdError + Send + Sync + 'static>
    Pemmican<S, E>
{
    /// Create a new pemmican server instance
    pub fn new(config: Config, initial_state: S) -> Pemmican<S, E> {
        Pemmican {
            routes: CHashMap::new(),
            pool: CpuPool::new(config.num_threads),
            config: config,
            state: initial_state,
        }
    }

    /// Add a route to the server.  Routes map a path and method onto a handler.
    /// Currently, all routes must be defined and added prior to running the server.
    pub fn add_route(&mut self, path: &str, method: Method, handler: Handler<S, E>)
    {
        self.routes.insert( (path.to_owned(),method), handler );
    }

    /// Run the server.  It will run until the `shutdown_signal` future completes.
    /// You can use futures::future::empty() to run forever.
    pub fn run<F>(self, addr: &str, shutdown_signal: F) -> Result<(), Error>
        where F: Future<Item = (), Error = ()>
    {
        let keep_alive = self.config.keep_alive;
        let shutdown_timeout = self.config.shutdown_timeout;

        let arcself = Arc::new(self);
        let addr = addr.parse()?;
        let mut server = Http::new()
            .keep_alive(keep_alive)
            .bind(&addr, move|| Ok(arcself.clone()))?;
        server.shutdown_timeout(shutdown_timeout);
        server.run_until(shutdown_signal).map_err(|e| From::from(e))
    }
}

impl<S: Send + Sync + 'static + Default,
     E: StdError + Send + Sync + 'static>
    Default for Pemmican<S, E>
{
    fn default() -> Self {
        Self::new(Config::default(), S::default())
    }
}

impl<S: Send + Sync + 'static,
     E: StdError + Send + Sync + 'static>
    Service for Pemmican<S, E>
{
    type Request = Request;
    type Response = Response;
    type Error = ::hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        // FIXME: we are cloning these due to HashMap get requiring a reference
        // and the issues with borrowing. Research ways to avoid this.
        let path = req.path().to_owned();
        let method: Method = req.method().clone();

        if let Some(handler) = self.routes.get_mut( &(path,method) ) {
            Box::new(
                // FIXME: once hyper deals with issue #1128 (slated for 0.12), rework
                // this code.
                (handler)(self, req).map_err(
                    |e| hyper::Error::Io(::std::io::Error::new(::std::io::ErrorKind::Other, e))
                )
            )
        } else {
            Box::new(future::ok(
                Response::new().with_status(StatusCode::NotFound)
            ))
        }
    }
}
