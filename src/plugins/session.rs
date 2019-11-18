
use futures::Future;
use plugins::{Plugin, PluginData};
use hyper::header::Cookie as CookieHeader;
use hyper::header::SetCookie;
use cookie::Cookie;
use textnonce::TextNonce;

header! { (Dnt, "Dnt") => [String] }
header! { (Tk, "Tk") => [String] }

/// This plugin implements sessions. Sessions associate subsequent requests with
/// earlier requests. Sessions are maintained automatically by always setting a
/// cookie initially and finding it again on subsequent requests.
///
/// This plugin only manages the cookie and maintains PluginData.session_id.
/// Associating data with that session_id is left up to the consumer of this library
/// (hint: Store it in your shared state, the S type parameter on Pemmican, perhaps
/// with a CHashMap)
///
/// Plug this in before main content handling plugins
pub struct Session {
    cookie_name: String,
    secure: bool,
    http_only: bool,
    respect_dnt_ad_absurdum: bool,
}

impl Session
{
    /// Create the Session plugin.
    ///
    /// `cookie_name` is the name of the cookie (e.g. PHP uses PHP_SESS_ID)
    ///
    /// `secure` is whether or not to allow transmission of the cookie over HTTP (without SSL)
    ///
    /// `http_only` is whether or not to restrict the cookie to the HTTP protocol (or else
    /// allow javascript to access it)
    pub fn new(cookie_name: String, secure: bool, http_only: bool) -> Session {
        Session {
            cookie_name: cookie_name,
            secure: secure,
            http_only: http_only,
            respect_dnt_ad_absurdum: false,
        }
    }

    /// If you set this, then clients setting the "DNT: 1" HTTP header will be unable
    /// to get sessions (using a cookie and checking it later is, strictly speaking,
    /// tracking).
    pub fn respect_dnt_ad_absurdum(&mut self) {
        self.respect_dnt_ad_absurdum = true;
    }
}

impl<S,E> Plugin<S,E> for Session
    where S: 'static, E: 'static
{
    fn handle(&self, mut data: PluginData<S>)
              -> Box<dyn Future<Item = PluginData<S>, Error = E>>
    {
        if self.respect_dnt_ad_absurdum {
            // Respect Dnt
            let mut dnt = false;
            if let Some(header) = data.request.headers().get::<Dnt>() {
                match *header {
                    Dnt(ref s) => {
                        if &*s != "0" {
                            dnt = true;
                        }
                    },
                }
            }
            if dnt {
                // The user has requested Do Not Track.  We strictly comply by removing
                // any existing session and refusing to start one while this header is
                // present
                data.session_id = None;

                // Set the Tk header, informing them that we are not tracking
                data.response.headers_mut().set(Tk("N".to_owned()));

                return Box::new(::futures::future::ok(data));
            }
        }

        let mut maybe_key: Option<String> = None;
        if let Some(cookie_header) = data.request.headers().get::<CookieHeader>() {
            if let Some(cookie_value) = cookie_header.get(&*self.cookie_name) {
                maybe_key = Some(cookie_value.to_owned());
            }
        }

        if let Some(key) = maybe_key {
            // Associate existing session
            data.session_id = Some(key.to_owned());
            return Box::new(::futures::future::ok(data));
        }

        // Create new session
        let key = TextNonce::new().into_string();
        data.session_id = Some(key.clone());

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
