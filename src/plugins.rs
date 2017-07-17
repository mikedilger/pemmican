
use super::Shared;
use std::sync::Arc;
use futures::Future;
use hyper::server::{Request, Response};


pub struct PluginData<S: Send + Sync>
{
    pub shared: Arc<Shared<S>>,
    pub request: Request,
    pub response: Response,
}


/// A plugable component, run on every request (for which a valid route exists)
pub trait Plugin<S: Send + Sync + 'static, E: 'static>: Send + Sync
{
    /// This runs before the handler, and potentially modifies the request
    fn before_handler(&self, request: &mut Request);

    /// This runs after the handler. This must return a future that uses the passed in
    /// future, returning the modified future, (e.g. perhaps using and_then)
    /// This potentially modifies the Response (available through the `data` parameter).
    fn after_handler(&self,
                     future: Box<Future<Item = PluginData<S>, Error = E>>)
                     -> Box<Future<Item = PluginData<S>, Error = E>>;
}

impl<'a, S: Send + Sync + 'static, E: 'static, T: Send + Sync + Plugin<S,E>>
    Plugin<S, E> for &'a T
{
    fn before_handler(&self, request: &mut Request)
    {
        (*self).before_handler(request)
    }

    fn after_handler(&self,
                     future: Box<Future<Item = PluginData<S>, Error = E>>)
                     -> Box<Future<Item = PluginData<S>, Error = E>>
    {
        (*self).after_handler(future)
    }
}
