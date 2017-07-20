# Pemmican

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)
[![Apache-2.0 licensed](https://img.shields.io/badge/license-APACHE2-blue.svg)](./LICENSE-APACHE)

[pemmican on crates.io](https://crates.io/crates/pemmican)

[Documentation](https://docs.rs/pemmican)

Pemmican is a Web server library built in top of [hyper](https://hyper.rs) for
the Rust language.

## Overview

### Pemmican is fast

Rust is a systems level language, and runs at about the same speed as the
fastest compiled languages such as C and C++, with careful coding.

We attempt to minimize memory copies and any other source of slowness.

### Pemmican is parallel

Pemmican provides a thread pool, so you can run each page handler on an existing
thread, and run them in parallel.

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

### Pemmican is modular and configurable

Pemmican is a rust library. It is generic, and does not define your website.
You define routes and add them dynamically.

Pemmican supports plugins, so that functionality can be added via separate crates,
and more importantly so that functionality you don't want can be left out.

Pemmican lets you configure various settings of the libraries it depends on,
rather than choosing for you. This includes (as of this writing) `num_threads`,
`shutdown_timeout`, and `keep_alive`.

### Pemmican uses your error type

Pemmican lets you define the Error type you will use, as long as it is
`Error + Send + Sync + 'static`.

### Pemmican lets you share state

Pemmican lets you share global state between your handlers, as long as it is
`Send + Sync + 'static`.

## Caveats and Warnings

Pemmican is very new (it was started in June 2017), and is likely to undergo substantial
changes. If you use it now, expect breaking changes.

The plugin architecture is especially new, and very likely to break multiple times before
it settles down.

## Example

[Basic Example](tests/basic.rs)

[Example using ThreadPool](tests/slow.rs)

[Example using a Plugin](tests/plugin.rs)

## Other similar crates

Other authors are also working towards similar goals.  Have a look at the
following projects which also use asynchronous hyper to provide a web
services:

* [zircon](https://crates.io/crates/zicron)
* [backtalk](https://crates.io/crates/backtalk)
* [pronghorn](https://crates.io/crates/pronghorn)
* [jsonrpc-http-server](https://crates.io/crates/jsonrpc-http-server)
* As of July 2017, that is about it (there are 37 crates that depend on hyper 0.11,
  and it seems only these 4 try to be a web server)

Additionally consider these more mature frameworks running on synchronous
hyper:

* [iron](https://crates.io/crates/iron)
* [rocket](https://crates.io/crates/rocket)
* I'm sure there are others that I don't know about.
