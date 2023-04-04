use rune::runtime::Protocol;
use rune::{Any, ContextError, Module};
use std::fmt;
use std::fmt::Write;
use std::sync::Arc;

/// Construct the `http` module.
pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("http");

    module.ty::<Agent>()?;
    module.ty::<Request>()?;
    module.ty::<Response>()?;
    module.ty::<Error>()?;

    module.function(["Agent", "new"], Agent::new)?;
    module.inst_fn("get", Agent::get)?;
    module.inst_fn("put", Agent::put)?;
    module.inst_fn("post", Agent::post)?;
    module.inst_fn("delete", Agent::delete)?;

    module.inst_fn("call", Request::call)?;
    module.inst_fn("set", Request::set)?;
    module.inst_fn("send_string", Request::send_string)?;

    module.inst_fn("status", Response::status)?;
    module.inst_fn("status_text", Response::status_text)?;
    module.inst_fn("into_string", Response::into_string)?;
    module.inst_fn("into_file", Response::into_file)?;

    module.inst_fn(Protocol::STRING_DISPLAY, Error::display)?;
    Ok(module)
}

#[derive(Debug, Any)]
pub struct Agent {
    client: ureq::Agent,
}

#[derive(Debug, Any)]
pub struct Request {
    request: ureq::Request,
}

#[derive(Debug, Any)]
pub struct Response {
    response: ureq::Response,
}

#[derive(Debug, Any)]
pub struct Error {
    error: ureq::Error,
}

impl Agent {
    pub fn new() -> Self {
        Self {
            client: ureq::builder()
                .tls_config(Arc::new(crate::TLS_CONFIG.clone()))
                .build(),
        }
    }

    pub fn get(&self, url: &str) -> Request {
        Request {
            request: self.client.get(url),
        }
    }

    pub fn put(&self, url: &str) -> Request {
        Request {
            request: self.client.put(url),
        }
    }

    pub fn post(&self, url: &str) -> Request {
        Request {
            request: self.client.post(url),
        }
    }

    pub fn delete(&self, url: &str) -> Request {
        Request {
            request: self.client.delete(url),
        }
    }
}

impl Request {
    pub fn call(self) -> Result<Response, Error> {
        Ok(Response {
            response: self.request.call().map_err(|e| Error { error: e })?,
        })
    }

    pub fn set(self, key: &str, value: &str) -> Self {
        Request {
            request: self.request.set(key, value),
        }
    }

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
    pub fn status(&self) -> u16 {
        self.response.status()
    }

    pub fn status_text(&self) -> String {
        self.response.status_text().to_string()
    }

    pub fn into_string(self) -> Result<String, Error> {
        self.response
            .into_string()
            .map_err(|e| Error { error: e.into() })
    }

    pub fn into_file(self, file: &str) -> Result<(), Error> {
        let mut file = std::fs::File::create(file).map_err(|e| Error { error: e.into() })?;
        std::io::copy(&mut self.response.into_reader(), &mut file)
            .map_err(|e| Error { error: e.into() })?;
        Ok(())
    }
}

impl Error {
    pub fn display(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "{}", self.error)
    }
}
