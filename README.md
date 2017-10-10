# Pemmican

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)
[![Apache-2.0 licensed](https://img.shields.io/badge/license-APACHE2-blue.svg)](./LICENSE-APACHE)

[pemmican on crates.io](https://crates.io/crates/pemmican)

[Documentation](https://docs.rs/pemmican)

Pemmican is a Web server library built in top of [hyper](https://hyper.rs) for
the Rust language.

## Caveats and Warnings

Pemmican is still rather new (it was started in June 2017), and is likely to undergo
substantial changes. If you use it now, expect breaking changes. We expect it won't be
generally useful until at least version 0.4.

The plugin architecture is especially new, and very likely to break when we fix
issue #8.

Currently, request and response bodies must fit entirely in memory as they are not
streamed.  This means pemmican is currently unsuitable for file uploads, for example.
We intend to fix this in version 0.4, but we may need to wait for hyper 0.12 before
we release 0.4.

## Overview

Pemmican is (or should be, once the issues are fixed)
* fast (rust)
* parallel (thread pool)
* asynchronous (hyper 0.11 w/ futures)
* modular (plugins)
* not opinionated (configurable and general purpose)
* lets you share global state

### Pemmican is fast

Pemmican is written in rust, a systems level language that generates code that typically
runs at about the same speed as the other fastest compiled languages available (such as C
and C++).

We attempt to minimize memory allocation, eschew blocking I/O, and try to use the fastest
algorithms available.  This is an ongoing effort.

Very basic performance testing is getting me 9,633 requests per second using only 28% of
the CPUs on an Intel(R) Xeon(R) CPU E3-1246 v3 @ 3.50GHz running Linux 4.12.13-1-ARCH.
These were not real-world requests, but ideallized localhost requests serving a static
"Hello World!" page handler.  This shows the core pathway is reasonably fast.

Other non-core pathways are not fully optimized.

### Pemmican is parallel

Pemmican provides a thread pool. You can choose to run your page handler logic on a
pre-existing thread from the thread pool.

### Pemmican is asynchronous

Pemmican uses the new asynchonous version of hyper, (based on tokio, futures,
mio, etc). This means that whenever a task is unable to continue right away,
your processor cores can move on to something else, keeping them busy whenever
any task is able to progress, rather than sitting idle waiting for an event.

In order to keep your cores busy whenever work becomes available, you must
be sure that the Future your page handlers return is not calling into blocking I/O
functions.  If any of them are, you can still use pemmican to great effect but
you probably want to configure a much larger thread pool.  If you never call
a blocking I/O call, then theoretically you should get maximum performace with
one thread per core.

### Pemmican is modular

Pemmican is a rust library. It is generic, and does not define your website.
You define routes and add them dynamically.

Pemmican supports plugins, so that functionality can be added via separate crates,
and more importantly so that functionality you don't want can be left out.

### Pemmican is not opinionated

Pemmican lets you configure various settings of the libraries it depends on,
rather than choosing for you. This includes (as of this writing) `num_threads`,
`shutdown_timeout`, and `keep_alive`.  We aim to be as configurable as possible,
only making choices when we absolutely must.

Pemmican lets you define the Error type you will use, as long as it is
`Error + Send + Sync + 'static`.

### Pemmican lets you share global state

Pemmican lets you share global state between your handlers, as long as it is
`Send + Sync + 'static`.

## Example

[Basic Example](tests/basic.rs)

[Example using ThreadPool](tests/slow.rs)

[Example using a Plugin](tests/plugin.rs)

[Example using DynamicRouter](tests/dynamic.rs)

[Example using Htdocs](tests/htdocs.rs)

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

There is also tk-http, which is async http without hyper:

* [tk-http](https://github.com/swindon-rs/tk-http)

Additionally consider these more mature frameworks running on synchronous
hyper:

* [iron](https://crates.io/crates/iron)
* [rocket](https://crates.io/crates/rocket)
* I'm sure there are others that I don't know about.
