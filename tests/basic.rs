
extern crate pemmican;
extern crate hyper;
extern crate futures;

use std::io::Error as IoError;
use std::sync::Arc;
use futures::Future;
use hyper::{Method, StatusCode};
use pemmican::{Pemmican, Config, PluginData, Plugin};

// This is the static router
struct MyRouter;
impl Plugin<(),IoError> for MyRouter {
    fn handle(&self, mut data: PluginData<()>)
              -> Box<dyn Future<Item = PluginData<()>, Error = IoError>>
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
fn home(mut data: PluginData<()>)
        -> Box<dyn Future<Item = PluginData<()>, Error = IoError>>
{
    data.response.set_body(format!("Hello World!"));
    data.response.set_status(StatusCode::Ok);
    Box::new(futures::future::ok( data ))
}

#[test]
fn main()
{
    // Create pemmican
    let pemmican = Pemmican::new(
        Config::default(),
        vec![Arc::new(Box::new(MyRouter))], // <-- plug in the router
        ()
    );

    // And run the server
    let _ = pemmican.run("127.0.0.1:3000",
                         //futures::future::empty() // this runs indefinately
                         futures::future::ok(()) // this completes immediately
    );
}
