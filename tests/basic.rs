
extern crate pemmican;
extern crate hyper;
extern crate futures;

use pemmican::{Pemmican, Config};
use hyper::server::{Request, Response};
use hyper::Method;
use futures::Future;

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
    // Create pemmican
    let mut pemmican = Pemmican::new( Config::default(), () );

    // Setup our route
    pemmican.add_route("/", Method::Get, home);

    // And run the server
    let _ = pemmican.run("127.0.0.1:3000",
                         //futures::future::empty() // this runs indefinately
                         futures::future::ok(()) // this completes immediately
    );
}
