
extern crate pemmican;
extern crate hyper;
extern crate futures;

use pemmican::{Pemmican, Config, Router, Handler};
use hyper::server::{Request, Response};
use hyper::Method;
use futures::Future;

// Define and implement a static router
struct MyRouter;
impl Router<(), ::std::io::Error> for MyRouter
{
    fn get_handler(&self, path: &str, method: &Method) -> Option<Handler<(), ::std::io::Error>> {
        match (path, method) {
            ("/", &Method::Get) => Some(home),
            _ => None,
        }
    }
}

// This is our home page handler
fn home(_pemmican: &Pemmican<(), ::std::io::Error>, _request: &Request)
         -> Box<dyn Future<Item = Response, Error = ::std::io::Error>>
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
    let pemmican = Pemmican::new( Config::default(), Box::new(MyRouter), () );

    // And run the server
    let _ = pemmican.run("127.0.0.1:3000",
                         //futures::future::empty() // this runs indefinately
                         futures::future::ok(()) // this completes immediately
    );
}
