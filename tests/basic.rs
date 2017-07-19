
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

fn greet(pemmican: &Pemmican<State, ::std::io::Error>, request: &Request)
         -> Box<Future<Item = Response, Error = ::std::io::Error>>
{
    Box::new(
        futures::future::ok(
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

fn slow(pemmican: &Pemmican<State, ::std::io::Error>, _request: &Request)
        -> Box<Future<Item = Response, Error = ::std::io::Error>>
{
    Box::new(
        pemmican.shared.pool.spawn_fn(|| Ok( {
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
    let page_visits = Arc::new(PageVisits::new());

    let mut pemmican = Pemmican::new(
        Config::default(),
        State {
            page_visits: page_visits.clone(),
        }
    );

    pemmican.plug_in( page_visits.clone() );

    pemmican.add_route("/", Method::Get, greet);
    pemmican.add_route("/slow", Method::Get, slow);

    let _ = pemmican.run("127.0.0.1:3000",
                         //futures::future::empty() // this runs indefinately
                         futures::future::ok(()) // this completes immediately
    );
}
