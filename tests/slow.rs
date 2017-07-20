
extern crate pemmican;
extern crate hyper;
extern crate futures;

use pemmican::{Pemmican, Config};
use hyper::server::{Request, Response};
use hyper::Method;
use futures::Future;

// This is our home page handler
fn home(pemmican: &Pemmican<(), ::std::io::Error>, _request: &Request)
        -> Box<Future<Item = Response, Error = ::std::io::Error>>
{
    Box::new(
        pemmican.shared.pool.spawn_fn(|| Ok( {
            // NOTE: this is not asynchronous programming! This call blocks. However, we
            // are using it as a proxy for an actual non-blocking but long-running task,
            // for example purposes only.
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
