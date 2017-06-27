
#![feature(integer_atomics)]

extern crate pemmican;
extern crate hyper;
extern crate futures;

use std::sync::atomic::{AtomicU64, Ordering};
use pemmican::{Pemmican, Handler};
use hyper::server::{Request, Response};
use hyper::Method;
use futures::Future;

struct Greet {
    count: AtomicU64,
}

impl Handler for Greet {
    fn run(&self, _pemmican: &Pemmican, _request: Request)
           -> Box<Future<Item = Response, Error = hyper::Error>>
    {
        Box::new(
            futures::future::ok(
                Response::new().with_body(
                    format!("This page has been accessed {} times.\n",
                            self.count.fetch_add(1, Ordering::SeqCst) + 1))))
    }
}

struct Slow;

impl Handler for Slow {
    fn run(&self, pemmican: &Pemmican, _request: Request)
        -> Box<Future<Item = Response, Error = hyper::Error>>
    {
        Box::new(
            pemmican.pool.spawn_fn(|| Ok( {
                ::std::thread::sleep(::std::time::Duration::from_secs(3));
                "This response delayed 3 seconds.\n".to_owned()
            })).map(|x| {
                Response::new().with_body(x)
            })
        )
    }
}

#[test]
fn main()
{
    let mut pemmican = Pemmican::new();
    pemmican.add_route("/", Method::Get, Greet {
        count: AtomicU64::new(0)
    });
    pemmican.add_route("/slow", Method::Get, Slow);
    let _ = pemmican.run("127.0.0.1:3000",
                         futures::future::ok(()) // so that it completes immediately
    );
}
