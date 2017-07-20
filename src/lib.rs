#![doc(html_root_url = "https://docs.rs/pemmican")]

//! Pemmican is a Web server library built on top of hyper for the Rust language.
//!
//! Introductory documentation is at https://github.com/mikedilger/pemmican

extern crate hyper;
extern crate futures;
extern crate futures_cpupool;
extern crate chashmap;

pub mod error;
pub use error::Error;

pub mod router;
pub use router::{Router, DynamicRouter};

pub mod plugins;
pub use plugins::{Plugin, PluginData};

use futures::Future;
use futures::future;
use futures_cpupool::CpuPool;
use hyper::server::{Http, Request, Response, Service};
use hyper::StatusCode;
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
/// to the services it provides (such as access to your state S and access to the
/// thread pool).
pub type Handler<S, E> =
    fn(pemmican: &Pemmican<S, E>, &Request)
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

/// A Shared component within Pemmican, accessible to plugins
pub struct Shared<S: Send + Sync>
{
    pub pool: CpuPool,
    #[allow(dead_code)] // this is provided for handlers; this library does not use it
    pub state: S,
}

/// A Pemmican server instance.
pub struct Pemmican<S: Send + Sync, E>
{
    router: Box<Router<S, E>>,
    pub config: Config,
    plugins: Vec<Arc<Plugin<S, E>>>,
    pub shared: Arc<Shared<S>>,
}

impl<S, E> Pemmican<S, E>
    where S: Send + Sync + 'static,
          E: Send + Sync + StdError + 'static
{
    /// Create a new pemmican server instance
    pub fn new(config: Config, router: Box<Router<S, E>>, initial_state: S)
               -> Pemmican<S, E>
    {
        let num_threads = config.num_threads;
        Pemmican {
            router: router,
            config: config,
            plugins: Vec::new(),
            shared: Arc::new(Shared {
                pool: CpuPool::new(num_threads),
                state: initial_state,
            })
        }
    }

    /// Plug in a plugin.
    /// Currently we have not yet implemented methods to order or re-order these.
    pub fn plug_in(&mut self, plugin: Arc<Plugin<S, E>>)
    {
        self.plugins.push(plugin);
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
        server.run_until(shutdown_signal).map_err(From::from)
    }
}

impl<S, E> Service for Pemmican<S, E>
    where S: Send + Sync + 'static,
          E: Send + Sync + StdError + 'static
{
    type Request = Request;
    type Response = Response;
    type Error = ::hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, mut req: Request) -> Self::Future {

        if let Some(handler) = self.router.get_handler(req.path(), req.method()) {
            // Run plugins before_handlers
            for m in &self.plugins {
                m.before_handler(&mut req);
            }

            let shared = self.shared.clone();

            // Call the handler
            let mut fut: Box<Future<Item = PluginData<S>, Error = E>> =
                Box::new(
                    (handler)(self, &req)
                        .map(move |response| {
                            PluginData {
                                shared: shared,
                                request: req,
                                response: response,
                            }
                        })
                );

            // Call each plugin, modifying the future each time
            for m in &self.plugins {
                fut = Box::new( m.after_handler(fut) );
            }

            // Map the future back to just a response
            let fut = Box::new( fut.map(|plugin_data| plugin_data.response) );
            // Map the error back to hyper error
            // FIXME: once hyper deals with issue #1128 (slated for 0.12), rework
            // this code.
            Box::new( fut.map_err(|e| {
                hyper::Error::Io(::std::io::Error::new(::std::io::ErrorKind::Other, e))
            }))
        } else {
            Box::new(future::ok(
                Response::new().with_status(StatusCode::NotFound)
            ))
        }
    }
}
