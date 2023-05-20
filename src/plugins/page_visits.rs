
use futures::future::Future;
use hyper::server::Request;
use chashmap::CHashMap;
//use ::Shared;
use ::plugins::{Plugin, PluginData};

#[derive(Default)]
pub struct PageVisits {
    counts: CHashMap<String, u32>
}

impl PageVisits {
    pub fn new() -> PageVisits
    {
        PageVisits {
            counts: CHashMap::new()
        }
    }

    pub fn get(&self, url: &str) -> Option<u32>
    {
        let url = url.to_owned();
        self.counts.get(&url).map(|guard| *guard)
    }
}

impl<S: Send + Sync + 'static, E: 'static> Plugin<S,E> for PageVisits {
    fn before_handler(&self, request: &mut Request) {
        let url: String = request.uri().as_ref().to_owned();
        if let Some(mut count) = self.counts.get_mut(&url) {
            *count += 1;
        } else {
            self.counts.insert(url, 1);
        }
    }

    fn after_handler(&self,
                     future: Box<dyn Future<Item = PluginData<S>, Error = E>>)
                     -> Box<dyn Future<Item = PluginData<S>, Error = E>>
    {
        // We do nothing here
        future
    }
}
