
extern crate pemmican;
extern crate hyper;
extern crate futures;
extern crate chashmap;

use std::io::Error as IoError;
use std::sync::Arc;
use futures::Future;
use hyper::{Method, StatusCode};
use pemmican::{Pemmican, Config, PluginData, Plugin};
use pemmican::plugins::PageVisits;

struct State {
    page_visits: Arc<PageVisits>,
}

// This is the static router
struct MyRouter;
impl Plugin<State,IoError> for MyRouter {
    fn handle(&self, mut data: PluginData<State>)
              -> Box<Future<Item = PluginData<State>, Error = IoError>>
    {
        match (data.request.path(), data.request.method()) {
            ("/", &Method::Get) => home(data),
            _ => {
                data.response.set_status(StatusCode::NotFound);
                Box::new(futures::future::ok( data ))
            }
        }
    }
}

// This is our home page handler
fn home(mut data: PluginData<State>)
        -> Box<Future<Item = PluginData<State>, Error = IoError>>
{
    // Here we access the plugin in the shared state object
    if let Some(c) = data.shared.state.page_visits.get( data.request.uri().as_ref() )
    {
        data.response.set_body(
            format!("This page has been accessed {} times.\n", c));
    } else {
        data.response.set_body(
            format!("We dont know how many times this page has been accessed.\n"));
    }

    Box::new( futures::future::ok( data ) )
}

#[test]
fn main()
{
    // Create the plugin.  We will have two references via an Arc: one to pass
    // to pemmican as the plugin, and the other so that we can query the page
    // count.
    let page_visits = Arc::new(PageVisits::new());

    // Create the shared state object
    let state = State {
        // Store a reference to the plugin within the shared state object, so we can
        // access it from our handler
        page_visits: page_visits.clone()
    };

    // Create pemmican, giving it the shared state object
    // NOTE: We are passing in two plugins: MyRouter and the 'page_visits' plugin.
    //       That is how you "plug in" a plugin.
    let pemmican = Pemmican::new(
        Config::default(),
        vec![
            Arc::new(Box::new(page_visits)),
            Arc::new(Box::new(MyRouter))
        ],
        state
    );

    // And run the server
    let _ = pemmican.run("127.0.0.1:3000",
                         //futures::future::empty() // this runs indefinately
                         futures::future::ok(()) // this completes immediately
    );
}
