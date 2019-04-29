
use std::path::{PathBuf, Component};
use std::fs::File;
use std::io::Read;
use futures::Future;
use hyper::{Method, StatusCode};
use hyper::header::CONTENT_LENGTH;
use plugins::{Plugin, PluginData};

/// This plugin serves static files from a document root.
///
/// If a previous router handler has already set a body, this plugin
/// will take no action.  It will only serve files if the path matches and
/// the response body has not yet been set.
pub struct Htdocs {
    docroot: PathBuf,
    index: Option<String>,
}

impl Htdocs {
    /// Create a new Htdocs plugin with the given document root.
    ///
    /// `index` is the file to search for in case a directory is specified,
    /// e.g. Some(`index.html`) or simply None if directories are not to be
    /// matched.
    pub fn new<P>(docroot: P, index: Option<String>) -> Htdocs
        where PathBuf: From<P>
    {
        Htdocs {
            docroot: From::from(docroot),
            index: index,
        }
    }
}

impl<S,E> Plugin<S,E> for Htdocs
    where S: Send + Sync + 'static,
          E: Send + 'static
{
    fn handle(&self, mut data: PluginData<S>)
        -> Box<Future<Item = PluginData<S>, Error = E>>
    {
        // Only handle GET and HEAD requests
        match data.request.method() {
            &Method::Get | &Method::Head => { },
            _ => return Box::new(::futures::future::ok(data)),
        }

        let mut filepath = {
            let input: PathBuf = From::from(data.request.path());
            let mut output: PathBuf = PathBuf::new();

            // Remove bad path components (all component except normal ones)
            for component in input.components() {
                match component {
                    Component::Normal(osstr) => output.push(osstr),
                    _ => {},
                }
            }

            self.docroot.join(output)
        };

        // The above work was not dependent on blocking calls. However, reading the file
        // is, and so we do that work within the threadpool
        let shared = data.shared.clone();
        let index = self.index.clone();
        Box::new(
            shared.pool.spawn_fn(move|| {

                if filepath.is_dir() {
                    if let Some(ref index) = index {
                        filepath.push(index);
                    } else {
                        // Do not handle directory requests if index is None
                        // (pass the data on to the next handler)
                        return Ok(data);
                    }
                }

                if filepath.exists() {
                    match File::open(&filepath) {
                        Err(e) => {
                            // File exists, but we cannot open it for some reason
                            warn!("Cannot open {:?}: {}", filepath, e);
                            data.response.set_status(StatusCode::InternalServerError);
                        },
                        Ok(mut f) => {
                            // FIXME -- We are reading the entire contents into memory at once.
                            //          This requires fixing, and is slated to be fixed in version
                            //          0.4 (hopefully).  See issue #8.
                            let mut buffer = Vec::new();
                            match f.read_to_end(&mut buffer) {
                                Ok(count) => {
                                    data.response.headers_mut().insert(CONTENT_LENGTH, count);
                                    if data.request.method() != &Method::Head {
                                        data.response.set_body(buffer);
                                    }
                                    data.response.set_status(StatusCode::Ok);
                                },
                                Err(e) => {
                                    // File cannot be read
                                    warn!("Cannot read {:?}: {}", filepath, e);
                                    data.response.set_status(StatusCode::InternalServerError);
                                }
                            };
                        }
                    }
                }

                Ok(data)
            })
        )
    }
}

