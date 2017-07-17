# Pemmican

Pemmican is a Web server library built in top of [hyper](https://hyper.rs) for
the Rust language.

## Overview

### Pemmican is fast

Rust is a systems level language, and runs at about the same speed as the
fastest compiled languages such as C and C++, with careful coding.

We attempt to minimize memory copies and any other source of slowness.

### Pemmican is asynchronous

Pemmican uses the new asynchonous version of hyper, (based on tokio, futures,
mio, etc). This means that whenever a task is unable to continue right away,
your processor cores can move on to something else, keeping them busy whenever
any task is able to progress.

    Synchronous blocking I/O (the simpler way) uses worker threads which will
    block whenever something is not ready, and eventually you may run out of
    threads, leaving your server idle, even when other tasks are ready to progress.
    Because any number of threads might be blocked, you must either spawn up a
    new thread for every new task (not efficient), or you need to accept that
    sometimes you will have work ready to be done, but no threads available to do
    it (also not efficient). Asynchronous I/O programming avoids these problems.

In order to keep your cores busy whenever work becomes available, you must
write your handler code to return futures, and you must be sure that your
handlers do not call blocking functions.  If you are successful in that,
you can run pemmican with one thread per hardware core, and know that it is
maximally efficient.

### Pemmican is modular

Pemmican is a rust library. It is generic, and does not define your website.
You define routes and add them dynamically.

Pemmican features will be developed modularly, so you can use as much or as
little as you wish.

### Example

```Rust
extern crate pemmican;
extern crate hyper;
extern crate futures;

use pemmican::{Pemmican, Config};
use hyper::server::{Request, Response};
use hyper::Method;
use futures::Future;
use std::io::Error as IoError;

// Here is a structure where you can share global state, accessible to your
// handler functions.  It must be `Send + Sync + 'static`
struct State;

// You can type-parameterize Pemmican with your State and any type that implements
// `Error + Send + Sync + 'static' for your error type.
fn home(pemmican: &Pemmican<State, IoError>, _request: Request)
  -> Box<Future<Item = Response, Error = IoError>>
{
  Box::new(
    futures::future::ok(
      Response::new().with_body(
        "Hello World!".to_owned())))
}

fn main()
{
  let mut pemmican = Pemmican::new(
    Config::default(),
    State
  );

  pemmican.add_route("/", Method::Get, home);

  let _ = pemmican.run("127.0.0.1:3000",
                       futures::future::empty());
}
```
