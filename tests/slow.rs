
extern crate pemmican;
extern crate hyper;
extern crate futures;

use std::io::Error as IoError;
use std::sync::Arc;
use futures::Future;
use hyper::{Method, StatusCode};
use pemmican::{Pemmican, Config, PluginData, Plugin};

// Define and implement a static router
struct MyRouter;
impl Plugin<(),IoError> for MyRouter {
    fn handle(&self, mut data: PluginData<()>)
              -> Box<Future<Item = PluginData<()>, Error = IoError>>
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
        -> Box<Future<Item = PluginData<()>, Error = IoError>>
{
    let shared = data.shared.clone();
    Box::new(
        shared.pool.spawn_fn(move || {
            // NOTE: this is not asynchronous programming! This call blocks. However, we
            // are using it as a proxy for an actual non-blocking but long-running task,
            // for example purposes only.
            ::std::thread::sleep(::std::time::Duration::from_secs(3));
            data.response.set_body(
                "This response delayed 3 seconds.\n".to_owned());
            Ok(data)
        }))
}

#[test]
fn main()
{
    // Create pemmican
    let pemmican = Pemmican::new(
        Config::default(),
        vec![Arc::new(Box::new(MyRouter))],
        ()
    );

    // And run the server
    let _ = pemmican.run("127.0.0.1:3000",
                         //futures::future::empty() // this runs indefinately
                         futures::future::ok(()) // this completes immediately
    );
}
