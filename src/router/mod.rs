
use std::error::Error as StdError;
use std::ops::Deref;
use hyper::Method;
use ::Handler;

mod dynamic_router;
pub use self::dynamic_router::DynamicRouter;

/// A trait for routing a URL path + a Method to a Handler
pub trait Router<S, E>: Send + Sync
    where S: Send + Sync + 'static, E: Send + Sync + StdError + 'static
{
    fn get_handler(&self, path: &str, method: &Method) -> Option<Handler<S, E>>;
}

// Anything that deferences into a Router also implements Router
impl<T, R, S, E> Router<S, E> for T
    where S: Send + Sync + 'static,
          E: Send + Sync + StdError + 'static,
          T: Deref<Target = R> + Send + Sync,
          R: Router<S, E> + ?Sized
{
    fn get_handler(&self, path: &str, method: &Method) -> Option<Handler<S, E>>
    {
        self.deref().get_handler(path, method)
    }
}
