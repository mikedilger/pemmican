
use std::sync::Arc;
use std::ops::Deref;
use futures::Future;
use hyper::server::{Request, Response};
use Shared;

pub struct PluginData<S>
{
    pub shared: Arc<Shared<S>>,
    pub request: Request,
    pub response: Response,
}

/// A plugin provides a handler for a request.
///
/// Code within these handlers should take care not to block or call
/// functions which may block. Instead, it should return futures immediately, or None.
///
/// In the case of an error, if at all possible, respond to the client with a 5xx
/// error code and return a Response rather than returning an Error through the
/// future.  However, either way works.
pub trait Plugin<S,E>
{
    fn handle(&self, data: PluginData<S>)
              -> Box<Future<Item = PluginData<S>, Error = E>>;
}

/// Anything that dereferences into a Plugin also implements Plugin
impl<S,E,R,T> Plugin<S,E> for T
    where T: Deref<Target = R>,
          R: Plugin<S,E>
{
    fn handle(&self, data: PluginData<S>)
              -> Box<Future<Item = PluginData<S>, Error = E>>
    {
        self.deref().handle(data)
    }
}


pub mod router;
pub use self::router::{Router, Handler};

pub mod page_visits;
pub use self::page_visits::PageVisits;

pub mod htdocs;
pub use self::htdocs::Htdocs;
