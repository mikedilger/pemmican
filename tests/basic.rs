
#![feature(integer_atomics)]

extern crate pemmican;
extern crate hyper;
extern crate futures;

use std::sync::atomic::{AtomicU64, Ordering};
use pemmican::{Pemmican, Config};
use hyper::server::{Request, Response};
use hyper::Method;
use futures::Future;

struct State {
    count: AtomicU64,
}


fn greet(pemmican: &Pemmican<State, ::std::io::Error>, _request: &Request)
         -> Box<Future<Item = Response, Error = ::std::io::Error>>
{
    Box::new(
        futures::future::ok(
            Response::new().with_body(
                format!("This page has been accessed {} times.\n",
                        pemmican.state.count.fetch_add(1, Ordering::SeqCst) + 1))))
}

fn slow(pemmican: &Pemmican<State, ::std::io::Error>, _request: &Request)
        -> Box<Future<Item = Response, Error = ::std::io::Error>>
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

#[test]
fn main()
{
    let mut pemmican = Pemmican::new(
        Config::default(),
        State {
            count: AtomicU64::new(0)
        }
    );

    pemmican.add_route("/", Method::Get, greet);
    pemmican.add_route("/slow", Method::Get, slow);

    let _ = pemmican.run("127.0.0.1:3000",
                         //futures::future::empty() // this runs indefinately
                         futures::future::ok(()) // this completes immediately
    );
}
