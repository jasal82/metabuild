use rune::runtime::{Formatter, VmResult};
use rune::{Any, ContextError, Module};
use rune::alloc::fmt::TryWrite;
use std::sync::Arc;

/// Construct the `http` module.
pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("http")?;

    module.ty::<Client>()?;
    module.ty::<Request>()?;
    module.ty::<Response>()?;
    module.ty::<Error>()?;

    module.function_meta(Client::new)?;
    module.function_meta(Client::get)?;
    module.function_meta(Client::put)?;
    module.function_meta(Client::post)?;
    module.function_meta(Client::delete)?;

    module.function_meta(Request::call)?;
    module.function_meta(Request::set)?;
    module.function_meta(Request::send_string)?;

    module.function_meta(Response::status)?;
    module.function_meta(Response::status_text)?;
    module.function_meta(Response::into_string)?;
    module.function_meta(Response::into_file)?;

    module.function_meta(Error::string_display)?;

    Ok(module)
}

#[derive(Debug, Any)]
#[rune(item = ::http)]
pub struct Client {
    client: ureq::Agent,
}

#[derive(Debug, Any)]
#[rune(item = ::http)]
pub struct Request {
    request: ureq::Request,
}

#[derive(Debug, Any)]
#[rune(item = ::http)]
pub struct Response {
    response: ureq::Response,
}

#[derive(Debug, Any)]
#[rune(item = ::http)]
pub struct Error {
    error: ureq::Error,
}

impl Client {
    #[rune::function(path = Self::new)]
    pub fn new() -> Self {
        Self {
            client: ureq::builder()
                .tls_config(Arc::new(crate::TLS_CONFIG.clone()))
                .build(),
        }
    }

    #[rune::function]
    pub fn get(&self, url: &str) -> Request {
        Request {
            request: self.client.get(url),
        }
    }

    #[rune::function]
    pub fn put(&self, url: &str) -> Request {
        Request {
            request: self.client.put(url),
        }
    }

    #[rune::function]
    pub fn post(&self, url: &str) -> Request {
        Request {
            request: self.client.post(url),
        }
    }

    #[rune::function]
    pub fn delete(&self, url: &str) -> Request {
        Request {
            request: self.client.delete(url),
        }
    }
}

impl Request {
    #[rune::function]
    pub fn call(self) -> Result<Response, Error> {
        Ok(Response {
            response: self.request.call().map_err(|e| Error { error: e })?,
        })
    }

    #[rune::function]
    pub fn set(self, key: &str, value: &str) -> Self {
        Request {
            request: self.request.set(key, value),
        }
    }

    #[rune::function]
    pub fn send_string(self, body: &str) -> Result<Response, Error> {
        Ok(Response {
            response: self
                .request
                .send_string(body)
                .map_err(|e| Error { error: e })?,
        })
    }
}

impl Response {
    #[rune::function]
    pub fn status(&self) -> u16 {
        self.response.status()
    }

    #[rune::function]
    pub fn status_text(&self) -> String {
        self.response.status_text().to_string()
    }

    #[rune::function]
    pub fn into_string(self) -> Result<String, Error> {
        self.response
            .into_string()
            .map_err(|e| Error { error: e.into() })
    }

    #[rune::function]
    pub fn into_file(self, file: &str) -> Result<(), Error> {
        let mut file = std::fs::File::create(file).map_err(|e| Error { error: e.into() })?;
        std::io::copy(&mut self.response.into_reader(), &mut file)
            .map_err(|e| Error { error: e.into() })?;
        Ok(())
    }
}

impl Error {
    #[rune::function(protocol = STRING_DISPLAY)]
    pub fn string_display(&self, f: &mut Formatter) -> VmResult<()> {
        rune::vm_write!(f, "{}", self.error);
        VmResult::Ok(())
    }
}
