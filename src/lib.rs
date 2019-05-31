#![doc(html_root_url = "https://docs.rs/pemmican")]

//! Pemmican is a Web server library built on top of hyper for the Rust language.
//!
//! Introductory documentation is at https://git.optcomp.nz/mikedilger/pemmican

extern crate futures;
extern crate futures_cpupool;
#[macro_use]
extern crate hyper;
extern crate http;
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

mod never;
use never::Never;

use std::error::Error as StdError;
use std::sync::Arc;
use futures::{Future, IntoFuture};
use hyper::{Request, Response, Body};
use hyper::server::Server;
use hyper::service::Service;
use http::response::Builder as ResponseBuilder;
use hyper::StatusCode;


/// A Pemmican server instance.
pub struct Pemmican<S, E>
    where S: Send + Sync
{
    config: Config,
    pub shared: Arc<Shared<S>>,
    pub plugins: Vec<Arc<Box<Plugin<S, E>>>>,
}

impl<S, E> Pemmican<S, E>
    where S: Send + Sync + 'static,
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

        let arcself = Arc::new(self);
        let addr = addr.parse()?;
        let mut server = Server::bind(&addr)
            .http1_keepalive(keep_alive)
            .serve(move|| self);

        let graceful = server.with_graceful_shutdown(shutdown_signal);
        hyper::rt::spawn(graceful);
        Ok(())
    }
}

impl<S,E> Service for Pemmican<S, E>
    where S: Send + Sync + 'static,
          E: Send + Sync + StdError + 'static
{
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Response<Self::ResBody>, Error = Self::Error>>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {

        let data = PluginData {
            shared: self.shared.clone(),
            request: req,
            response_builder: ResponseBuilder::new(),
            body: None,
            session_id: None,
        };

        // Start with an 'ok' future
        let mut fut: Box<Future<Item = PluginData<S>, Error = E>> =
            Box::new(futures::future::ok(data));

        // Run plugin handlers (let them modify the future)
        for plugin in &self.plugins {
            let plug = plugin.clone(); // TBD: try to avoid this clone
            fut = Box::new(
                fut.and_then(move|data| {
                    plug.handle(data)
                })
            );
        }

        // Map the future back to just a response
        let fut = Box::new( fut.map(|mut data| {
            data.response_builder.body(
                data.body.unwrap_or(Body::empty())
            ).unwrap() // FIXME
        }));

        // any errors that remain are logged and InternalServerError
        // is returned to the client
        Box::new( fut.or_else(|e| {
            error!("error: {}", e);
            let mut builder = ResponseBuilder::new();
            builder.status(StatusCode::INTERNAL_SERVER_ERROR);
            let response: Response<Self::ResBody> = builder.body(Body::empty()).unwrap();
            futures::future::ok(response)
        }))
    }
}

impl <S, E> IntoFuture for Pemmican<S, E>
    where S: Send + Sync
{
    type Future = futures::future::FutureResult<Self::Item, Self::Error>;
    type Item = Self;
    type Error = Never;
    fn into_future(self) -> Self::Future {
        futures::future::ok(self)
    }
}
