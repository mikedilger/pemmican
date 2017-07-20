
extern crate pemmican;
extern crate hyper;
extern crate futures;

use pemmican::{Pemmican, Config, DynamicRouter};
use hyper::server::{Request, Response};
use hyper::Method;
use futures::Future;

// This is our home page handler
fn home(_pemmican: &Pemmican<(), ::std::io::Error>, _request: &Request)
         -> Box<Future<Item = Response, Error = ::std::io::Error>>
{
    Box::new(
        futures::future::ok(
            Response::new().with_body(
                format!("Hello World!"))
        )
    )
}

#[test]
fn main()
{
    // Create our router
    let router = DynamicRouter::new();
    router.insert("/", Method::Get, home);

    // Create pemmican
    let pemmican = Pemmican::new( Config::default(), Box::new(router), () );

    // And run the server
    let _ = pemmican.run("127.0.0.1:3000",
                         //futures::future::empty() // this runs indefinately
                         futures::future::ok(()) // this completes immediately
    );
}
