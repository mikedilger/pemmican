
extern crate pemmican;
extern crate hyper;
extern crate futures;

use std::io::Error as IoError;
use std::sync::Arc;
use futures::Future;
use hyper::Method;
use hyper::server::Response;
use pemmican::{Pemmican, Config, PluginData};
use pemmican::plugins::{Router, Htdocs};

// This is our home page handler
fn home(mut data: PluginData<()>)
        -> Box<Future<Item = PluginData<()>, Error = IoError>>
{
    data.response = Response::new().with_body(format!("Hello World!"));
    Box::new(futures::future::ok( data ))
}

#[test]
fn main()
{
    // Create a dynamic router
    let my_router = Router::new();
    my_router.insert("/", Method::Get, home);

    // Create an htdocs handler (serves from current directory, no indexes)
    let htdocs = Htdocs::new(".", None);

    // Create pemmican
    let pemmican = Pemmican::new(
        Config::default(),
        vec![Arc::new(Box::new(my_router)), // Serve our pages first
             Arc::new(Box::new(htdocs))], // Fallback to htdocs
        ()
    );

    // And run the server
    let _ = pemmican.run("127.0.0.1:3000",
                         //futures::future::empty() // this runs indefinately
                         futures::future::ok(()) // this completes immediately
    );
}
