
extern crate pemmican;
extern crate hyper;
extern crate futures;

use std::sync::Arc;
use pemmican::{Pemmican, Config};
use pemmican::plugins::PageVisits;
use hyper::server::{Request, Response};
use hyper::Method;
use futures::Future;

struct State {
    page_visits: Arc<PageVisits>,
}

// This is our home page handler
fn home(pemmican: &Pemmican<State, ::std::io::Error>, request: &Request)
        -> Box<Future<Item = Response, Error = ::std::io::Error>>
{
    Box::new(
        futures::future::ok(
            // Here we access the plugin in the shared state object
            if let Some(c) = pemmican.shared.state.page_visits.get( request.uri().as_ref() )
            {
                Response::new().with_body(
                    format!("This page has been accessed {} times.\n", c))
            } else {
                Response::new().with_body(
                    format!("We dont know how many times this page has been accessed.\n"))
            }
        )
    )
}

#[test]
fn main()
{
    // Create the plugin
    let page_visits = Arc::new(PageVisits::new());

    // Create the shared state object
    let state = State {
        // Store a reference to the plugin within the shared state object, so we can
        // access it from our handler
        page_visits: page_visits.clone()
    };

    // Create pemmican, giving it the shared state object
    let mut pemmican = Pemmican::new( Config::default(), state );

    // Plug-in the plugin
    pemmican.plug_in( page_visits.clone() );

    // Setup our route
    pemmican.add_route("/", Method::Get, home);

    // And run the server
    let _ = pemmican.run("127.0.0.1:3000",
                         //futures::future::empty() // this runs indefinately
                         futures::future::ok(()) // this completes immediately
    );
}
