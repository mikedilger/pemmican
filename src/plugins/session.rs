
use futures::Future;
use chashmap::{CHashMap, ReadGuard, WriteGuard};
use plugins::{Plugin, PluginData};
use hyper::header::Cookie as CookieHeader;
use hyper::header::SetCookie;
use cookie::Cookie;
use textnonce::TextNonce;

/// This plugin implements sessions. Sessions associate subsequent requests with
/// earlier requests. Sessions are maintained automatically by always setting a
/// cookie initially and finding it again on subsequent requests.
///
/// Plug this in before main content handling plugins
pub struct Session<D: Default> {
    cookie_name: String,
    data: CHashMap<String, D>,
    secure: bool,
    http_only: bool,
}

impl<D> Session<D>
    where D: Default
{
    /// Create the Session plugin.
    ///
    /// `cookie_name` is the name of the cookie (e.g. PHP uses PHP_SESS_ID)
    ///
    /// `secure` is whether or not to allow transmission of the cookie over HTTP (without SSL)
    ///
    /// `http_only` is whether or not to restrict the cookie to the HTTP protocol (or else
    /// allow javascript to access it)
    pub fn new(cookie_name: String, secure: bool, http_only: bool) -> Session<D> {
        Session {
            cookie_name: cookie_name,
            data: CHashMap::new(),
            secure: secure,
            http_only: http_only,
        }
    }

    /// Get a reference to a session's data D.  Use `PluginData`s session_id for the
    /// current session (if any).
    pub fn get_ref<'a>(&'a self, session_id: &String) -> Option<ReadGuard<'a, String, D>> {
        self.data.get(session_id)
    }

    /// Get a mutable reference to a session's data D.  Use `PluginData`s session_id
    /// for the current session (if any).
    pub fn get_mut<'a>(&'a self, session_id: &String) -> Option<WriteGuard<'a, String, D>> {
        self.data.get_mut(session_id)
    }
}

impl<S,E,D> Plugin<S,E> for Session<D>
    where S: 'static, E: 'static, D: Default
{
    fn handle(&self, mut data: PluginData<S>)
              -> Box<Future<Item = PluginData<S>, Error = E>>
    {
        let mut maybe_key: Option<String> = None;
        if let Some(cookie_header) = data.request.headers().get::<CookieHeader>() {
            if let Some(cookie_value) = cookie_header.get(&*self.cookie_name) {
                maybe_key = Some(cookie_value.to_owned());
            }
        }

        if let Some(key) = maybe_key {
            if self.data.contains_key(&key) {
                // Associate existing session
                data.session_id = Some(key.to_owned());
                return Box::new(::futures::future::ok(data));
            }
        }

        // Create new session
        let key = TextNonce::new().into_string();

        // Create new session data
        self.data.insert(key.clone(), D::default());

        // Create the cookie
        let mut cookie = Cookie::new(self.cookie_name.clone(), key);
        // expiry defaults to 'on close'
        // max_age defaults to None
        cookie.set_path("/"); // force a root path
        cookie.set_secure(self.secure);
        cookie.set_http_only(self.http_only);

        // Set the cookie
        data.response.headers_mut().set(
            SetCookie(vec![ cookie.to_string() ]));

        // Pass data on through
        Box::new(::futures::future::ok(data))
    }
}
