
use futures::Future;
use plugins::{Plugin, PluginData};
use hyper::header::StrictTransportSecurity;
use hyper::header::ReferrerPolicy;
use hyper::header::ContentType;

header! { (ContentSecurityPolicy, "Content-Security-Policy") => [String] }
header! { (XContentTypeOptions, "X-Content-Type-Options") => [String] }
header! { (XFrameOptions, "X-Frame-Options") => [String] }
header! { (XXssProtection, "X-Xss-Protection") => [String] }

pub enum GcReferrer {
    NoReferrer,
    SameOrigin
}
impl Default for GcReferrer {
    fn default() -> GcReferrer {
        GcReferrer::NoReferrer
    }
}


/// Good Citizen is a plugin that helps you treat users of the website with
/// the utmost respect. It deals with issues surrounding privacy, security,
/// and usability.
///
/// This plugin should probably be added near the end of your plugin chain, unless
/// you want to override it on a per-page basis, in which case you'll need to plug
/// it in before your router.
pub struct GoodCitizen {
    strict_transport_security: Option<StrictTransportSecurity>,
    referrer_policy: Option<ReferrerPolicy>,
    content_security_policy: Option<String>,
    x_content_type_options: Option<String>,
    x_frame_options: Option<String>,
    x_xss_protection: Option<String>,
}

impl Default for GoodCitizen {
    fn default() -> GoodCitizen {
        GoodCitizen {
            strict_transport_security: Some(
                StrictTransportSecurity::including_subdomains(31536000u64)),
            referrer_policy: Some(ReferrerPolicy::NoReferrer),
            content_security_policy: Some("default-src https:; block-all-mixed-content; upgrade-insecure-requests;".to_owned()),
            x_content_type_options: Some("nosniff".to_owned()),
            x_frame_options: Some("DENY".to_owned()),
            x_xss_protection: Some("1; mode=block".to_owned()),
        }
    }
}

impl GoodCitizen {
    pub fn new() -> GoodCitizen {
        Default::default()
    }

    pub fn disable_strict_transport_security(&mut self) {
        self.strict_transport_security = None;
    }

    pub fn set_strict_transport_security(&mut self, sts: StrictTransportSecurity) {
        self.strict_transport_security = Some(sts);
    }

    pub fn disable_referrer_policy(&mut self) {
        self.referrer_policy = None;
    }

    pub fn set_referrer_policy(&mut self, rp: ReferrerPolicy) {
        self.referrer_policy = Some(rp);
    }

    pub fn disable_content_security_policy(&mut self) {
        self.content_security_policy = None;
    }

    pub fn set_content_security_policy(&mut self, csp: String) {
        self.content_security_policy = Some(csp);
    }

    pub fn disable_x_content_type_options(&mut self) {
        self.x_content_type_options = None;
    }

    pub fn set_x_content_type_options(&mut self, xcto: String) {
        self.x_content_type_options = Some(xcto);
    }

    pub fn disable_x_frame_options(&mut self) {
        self.x_frame_options = None;
    }

    pub fn set_x_frame_options(&mut self, xfo: String) {
        self.x_frame_options = Some(xfo);
    }

    pub fn disable_x_xss_protection(&mut self) {
        self.x_xss_protection = None;
    }

    pub fn set_x_xss_protection(&mut self, xss: String) {
        self.x_xss_protection = Some(xss);
    }
}


impl<S,E> Plugin<S,E> for GoodCitizen
    where S: 'static, E: 'static
{
    fn handle(&self, mut data: PluginData<S>)
              -> Box<Future<Item = PluginData<S>, Error = E>>
    {
        if let Some(ref sts) = self.strict_transport_security {
            data.response.headers_mut().set(sts.clone());
        }
        if let Some(ref rp) = self.referrer_policy {
            data.response.headers_mut().set(rp.clone());
        }
        if let Some(ref csp) = self.content_security_policy {
            data.response.headers_mut().set(ContentSecurityPolicy(csp.clone()));
        }
        if let Some(ref xcto) = self.x_content_type_options {
            data.response.headers_mut().set(XContentTypeOptions(xcto.clone()));
        }
        if let Some(ref xss) = self.x_xss_protection {
            data.response.headers_mut().set(XXssProtection(xss.clone()));
        }

        Box::new(::futures::future::ok(data))
    }
}
