
use std::ops::Deref;
use futures::Future;
use hyper::Method;
use chashmap::CHashMap;
use plugins::{Plugin, PluginData};

pub type Handler<S, E> = fn(data: PluginData<S>)
                            -> Box<dyn Future<Item = PluginData<S>, Error = E>>;

pub struct Router<S, E> {
    routes: CHashMap<(String, Method), Handler<S,E>>,
}

impl<S,E> Router<S,E> {
    pub fn new() -> Router<S, E> {
        Router {
            routes: CHashMap::new()
        }
    }

    /// Define a route (insert or replace)
    pub fn insert(&self, path: &str, method: Method, handler: Handler<S, E>) {
        self.routes.insert( (path.to_owned(), method), handler );
    }

    /// Remove a route
    pub fn remove(&self, path: &str, method: Method) {
        self.routes.remove( &(path.to_owned(), method) );
    }

    /// Remove all routes
    pub fn clear(&self) {
        self.routes.clear();
    }
}

impl<S,E> Plugin<S,E> for Router<S,E>
    where S: 'static,
          E: 'static
{
    fn handle(&self, data: PluginData<S>)
              -> Box<dyn Future<Item = PluginData<S>, Error = E>>
    {
        let path = data.request.path().to_owned();
        let method = data.request.method().clone();
        match self.routes.get_mut(&(path,method))
        {
            Some(guard) => {
                let h = guard.deref();
                (h)(data)
            },
            None => {
                Box::new(::futures::future::ok(data))
            }
        }
    }
}
