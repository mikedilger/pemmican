
extern crate hyper;
extern crate futures;
extern crate futures_cpupool;
extern crate chashmap;

use chashmap::CHashMap;
use futures::Future;
use futures::future;
use futures_cpupool::CpuPool;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};
use std::time::Duration;
use std::sync::Arc;

pub trait Handler: 'static + Send + Sync {
    fn run(&self, pemmican: &Pemmican, request: Request)
           -> Box<Future<Item = Response, Error = ::hyper::Error>>;
}

pub struct Pemmican {
    pub pool: CpuPool,
    routes: CHashMap<(String, Method), Box<Handler>>,
}

impl Pemmican {
    pub fn new() -> Pemmican {
        Pemmican {
            pool: CpuPool::new(4), // FIXME, config setting num_threads
            routes: CHashMap::new(),
        }
    }

    pub fn add_route<H: Handler>(&mut self, path: &str, method: Method, handler: H)
    {
        self.routes.insert( (path.to_owned(),method), Box::new(handler) );
    }

    pub fn run(self, addr: &str) -> Result<(), hyper::Error>
    {
        let arcself = Arc::new(self);
        let addr = addr.parse().unwrap(); // FIXME: when error type is generalized
        let mut server = Http::new()
            .keep_alive(true) // FIXME: config setting keep_alive
            .bind(&addr, move|| Ok(arcself.clone()))?;
        server.shutdown_timeout(Duration::from_secs(1)); // FIXME: config shutdown_timeout
        server.run_until(future::empty()) // FIXME: maybe add shutdown future parameter
    }
}

impl Default for Pemmican {
    fn default() -> Self {
        Self::new()
    }
}

impl Service for Pemmican {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error; // FIXME: generalize
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        // FIXME: we are cloning these due to HashMap get requiring a reference
        // and the issues with borrowing. Research ways to avoid this.
        let path = req.path().to_owned();
        let method: Method = req.method().clone();

        if let Some(handler) = self.routes.get_mut( &(path,method) ) {
            handler.run(self, req)
        } else {
            Box::new(future::ok(
                Response::new().with_status(StatusCode::NotFound)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
