
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

pub type Handler<State> =
    fn(pemmican: &Pemmican<State>, Request)
       -> Box<Future<Item = Response, Error = ::hyper::Error>>;

pub struct Pemmican<State: Send + Sync + 'static> {
    routes: CHashMap<(String, Method), Handler<State>>,
    pub pool: CpuPool,
    #[allow(dead_code)] // this is provided for handlers; this library does not use it
    pub state: State,
}

impl<State: Send + Sync + 'static> Pemmican<State> {
    pub fn new(initial_state: State) -> Pemmican<State> {
        Pemmican {
            routes: CHashMap::new(),
            pool: CpuPool::new(4), // FIXME, config setting num_threads
            state: initial_state,
        }
    }

    pub fn add_route(&mut self, path: &str, method: Method, handler: Handler<State>)
    {
        self.routes.insert( (path.to_owned(),method), handler );
    }

    pub fn run<F>(self, addr: &str, shutdown_signal: F) -> Result<(), Error>
        where F: Future<Item = (), Error = ()>
    {
        let arcself = Arc::new(self);
        let addr = addr.parse()?;
        let mut server = Http::new()
            .keep_alive(true) // FIXME: config setting keep_alive
            .bind(&addr, move|| Ok(arcself.clone()))?;
        server.shutdown_timeout(Duration::from_secs(1)); // FIXME: config shutdown_timeout
        server.run_until(shutdown_signal).map_err(|e| From::from(e))
    }
}

impl<State: Send + Sync + 'static + Default> Default for Pemmican<State> {
    fn default() -> Self {
        Self::new(State::default())
    }
}

impl<State: Send + Sync + 'static> Service for Pemmican<State> {
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
            (handler)(self, req)
        } else {
            Box::new(future::ok(
                Response::new().with_status(StatusCode::NotFound)
            ))
        }
    }
}
