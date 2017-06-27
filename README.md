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

Synchronous blocking I/O (the older/simpler way) has worker threads which
blocked whenever something was not ready.  Because any number of threads might
be blocked, you either to spawn up a new thread for every new task (not
efficient), or you needed to accept that sometimes you would have useful work
but no threads available to do it (also not efficient).

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

## THINGS TO FIX

* Automate Multithreadedness:  We are currently requiring the consumer
  handlers to use the thread pool, rather than running their future on it
  for them.
* Error type: hyper::Error is not right, we need to generalize in a way that
  the consumer can extend.
* TLS: see this thread: https://github.com/hyperium/hyper/issues/1025
  and nayato's example (which may be out of date)
* Config file needed:  Currently at least: num_threads, shutdown_timeout
* Possibly fix cloning in impl Service for Pemmican
* Extend
