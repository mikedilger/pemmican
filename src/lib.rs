#![doc(html_root_url = "https://docs.rs/pemmican")]

//! Pemmican is a Web server library built on top of hyper for the Rust language.
//!
//! Introductory documentation is at https://github.com/mikedilger/pemmican

extern crate futures;
extern crate futures_cpupool;
extern crate tokio_service;
extern crate hyper;
extern crate chashmap;
#[macro_use]
extern crate log;
extern crate cookie;
extern crate textnonce;

pub mod error;
pub use error::Error;

pub mod config;
pub use config::Config;

pub mod shared;
pub use shared::Shared;

pub mod plugins;
pub use plugins::{PluginData, Plugin};



use std::error::Error as StdError;
use std::sync::Arc;
use futures::Future;
use tokio_service::Service;
use hyper::server::{Http, Request, Response};
use hyper::StatusCode;


/// A Pemmican server instance.
pub struct Pemmican<S, E>
{
    config: Config,
    pub shared: Arc<Shared<S>>,
    pub plugins: Vec<Arc<Box<Plugin<S, E>>>>,
}

impl<S, E> Pemmican<S, E>
    where S: 'static,
          E: Send + Sync + StdError + 'static
{
    /// Create a new pemmican server instance
    pub fn new(config: Config,
               plugins: Vec<Arc<Box<Plugin<S, E>>>>,
               initial_state: S)
               -> Pemmican<S, E>
    {
        let num_threads = config.num_threads;
        Pemmican {
            config: config,
            plugins: plugins,
            shared: Arc::new(Shared::new(num_threads, initial_state)),
        }
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
    where S: 'static,
          E: Send + Sync + StdError + 'static
{
    type Request = Request;
    type Response = Response;
    type Error = ::hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {

        let data = PluginData {
            shared: self.shared.clone(),
            request: req,
            response: Response::new().with_status(StatusCode::NotFound),
            session_id: None,
        };

        let mut fut: Box<Future<Item = PluginData<S>, Error = E>> =
            Box::new(::futures::future::ok(data));

        // Run plugin handlers
        for plugin in &self.plugins {
            let plug = plugin.clone();
            fut = Box::new(
                fut.and_then(move|data| {
                    plug.handle(data)
                })
            );
        }

        // Map future back to just a response
        let fut = Box::new( fut.map(|data| data.response) );

        // any errors that remain are logged and InternalServerError
        // is returned to the client
        Box::new( fut.or_else(|e| {
            error!("error: {}", e);
            ::futures::future::ok(Response::new().with_status(StatusCode::InternalServerError))
        }))
    }
}
