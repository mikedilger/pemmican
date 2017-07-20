
use std::error::Error as StdError;
use chashmap::CHashMap;
use hyper::Method;
use ::Handler;
use router::Router;

/// This is a dynamic router.  You can modify routes at runtime.
#[derive(Default)]
pub struct DynamicRouter<S, E>
    where S: Send + Sync + 'static, E: Send + Sync + StdError + 'static
{
    routes: CHashMap<(String, Method), Handler<S, E>>,
}

impl<S, E> DynamicRouter<S, E>
    where S: Send + Sync + 'static, E: Send + Sync + StdError + 'static
{
    pub fn new() -> DynamicRouter<S, E> {
        DynamicRouter {
            routes: CHashMap::new()
        }
    }

    /// Define a route (insert or replace)
    pub fn insert(&self, path: &str, method: Method, handler: Handler<S, E>) {
        // FIXME: we are cloning the path due to HashMap lookup requirements.
        // Research ways to avoid this.
        self.routes.insert( (path.to_owned(), method), handler );
    }

    /// Remove a route
    pub fn remote(&self, path: &str, method: Method) {
        // FIXME: we are cloning the path due to HashMap lookup requirements.
        // Research ways to avoid this.
        self.routes.remove( &(path.to_owned(), method) );
    }

    /// Remove all routes
    pub fn clear(&self) {
        self.routes.clear();
    }
}

impl<S, E> Router<S, E> for DynamicRouter<S, E>
    where S: Send + Sync + 'static, E: Send + Sync + StdError + 'static
{
    fn get_handler(&self, path: &str, method: &Method) -> Option<Handler<S, E>>
    {
        // FIXME: we are cloning the path due to HashMap lookup requirements.
        // Research ways to avoid this.
        let path = path.to_owned();
        let method = method.clone();

        self.routes.get_mut( &(path, method) ).map(|guard| *guard)
    }
}
