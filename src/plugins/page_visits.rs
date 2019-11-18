
use futures::Future;
use chashmap::CHashMap;
use plugins::{Plugin, PluginData};

/// This plugin counts page visits.  It counts visits to every URL accessed,
/// whether the URL is valid or not.  This router can be placed anywhere in
/// the chain; it will not disturb the other routers/handlers.
pub struct PageVisits {
    counts: CHashMap<String, u32>,
}

impl PageVisits
{
    pub fn new()  -> PageVisits {
        PageVisits {
            counts: CHashMap::new(),
        }
    }

    /// This function gets the number of times the path was called.
    pub fn get(&self, url_path: &str) -> Option<u32>
    {
        let url_path = url_path.to_owned();
        self.counts.get(&url_path).map(|guard| *guard)
    }
}

impl<S,E> Plugin<S,E> for PageVisits
    where S: 'static, E: 'static
{
    fn handle(&self, data: PluginData<S>)
              -> Box<dyn Future<Item = PluginData<S>, Error = E>>
    {
        // Update the visit count
        let url_path: String = data.request.path().to_owned();
        if let Some(mut count) = self.counts.get_mut(&url_path) {
            *count += 1;
        } else {
            self.counts.insert(url_path, 1);
        }

        // Pass data on through
        Box::new(::futures::future::ok(data))
    }
}
