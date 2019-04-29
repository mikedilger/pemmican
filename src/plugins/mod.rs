
use std::sync::Arc;
use std::ops::Deref;
use futures::Future;
use hyper::{Request, Response, Body};
use http::response::Builder as ResponseBuilder;
use Shared;

pub struct PluginData<S>
    where S: Send + Sync
{
    pub shared: Arc<Shared<S>>,
    pub request: Request<Body>,
    pub response_builder: ResponseBuilder,
    pub body: Option<Body>,
    pub session_id: Option<String>,
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
    where S: Send + Sync
{
    fn handle(&self, data: PluginData<S>)
              -> Box<Future<Item = PluginData<S>, Error = E>>;
}

/// Anything that dereferences into a Plugin also implements Plugin
impl<S,E,R,T> Plugin<S,E> for T
    where T: Deref<Target = R>,
          R: Plugin<S,E>,
          S: Send + Sync
{
    fn handle(&self, data: PluginData<S>)
              -> Box<Future<Item = PluginData<S>, Error = E>>
    {
        self.deref().handle(data)
    }
}

/*
pub mod router;
pub use self::router::{Router, Handler};

pub mod page_visits;
pub use self::page_visits::PageVisits;

pub mod htdocs;
pub use self::htdocs::Htdocs;

pub mod session;
pub use self::session::Session;

pub mod good_citizen;
pub use self::good_citizen::GoodCitizen;
*/
