use rune::{Any, Module, Value, ContextError};
use rune::runtime::{Bytes, Protocol};
use std::fmt;
use std::fmt::Write;

/// Construct the `http` module.
pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("http");

    module.ty::<Client>()?;
    module.ty::<Response>()?;
    module.ty::<RequestBuilder>()?;
    module.ty::<StatusCode>()?;
    module.ty::<Error>()?;

    module.function(["Client", "new"], Client::new)?;
    module.function(["get"], get)?;

    module.inst_fn("get", Client::get)?;
    module.inst_fn("post", Client::post)?;

    module.inst_fn("text", Response::text)?;
    module.inst_fn("json", Response::json)?;
    module.inst_fn("status", Response::status)?;

    module.inst_fn("send", RequestBuilder::send)?;
    module.inst_fn("header", RequestBuilder::header)?;
    module.inst_fn("body_bytes", RequestBuilder::body_bytes)?;

    module.inst_fn(Protocol::STRING_DISPLAY, Error::display)?;
    module.inst_fn(Protocol::STRING_DISPLAY, StatusCode::display)?;
    Ok(module)
}

#[derive(Debug, Any)]
pub struct Error {
    inner: reqwest::Error,
}

impl From<reqwest::Error> for Error {
    fn from(inner: reqwest::Error) -> Self {
        Self { inner }
    }
}

impl Error {
    fn display(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "{}", self.inner)
    }
}

#[derive(Debug, Any)]
struct Client {
    client: reqwest::blocking::Client,
}

#[derive(Debug, Any)]
pub struct Response {
    response: reqwest::blocking::Response,
}

#[derive(Debug, Any)]
pub struct StatusCode {
    inner: reqwest::StatusCode,
}

impl StatusCode {
    fn display(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "{}", self.inner)
    }
}

impl Response {
    fn text(self) -> Result<String, Error> {
        let text = self.response.text()?;
        Ok(text)
    }

    fn json(self) -> Result<Value, Error> {
        let text = self.response.json()?;
        Ok(text)
    }

    /// Get the status code of the response.
    fn status(&self) -> StatusCode {
        let inner = self.response.status();

        StatusCode { inner }
    }
}

#[derive(Debug, Any)]
pub struct RequestBuilder {
    request: reqwest::blocking::RequestBuilder,
}

impl RequestBuilder {
    /// Send the request being built.
    fn send(self) -> Result<Response, Error> {
        println!("Calling send");
        let response = self.request.send()?;
        println!("Got response");
        Ok(Response { response })
    }

    /// Modify a header in the request.
    fn header(self, key: &str, value: &str) -> Self {
        Self {
            request: self.request.header(key, value),
        }
    }

    /// Set the request body from bytes.
    fn body_bytes(self, bytes: Bytes) -> Result<Self, Error> {
        let bytes = bytes.into_vec();

        Ok(Self {
            request: self.request.body(bytes),
        })
    }
}

impl Client {
    fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Construct a builder to GET the given URL.
    fn get(&self, url: &str) -> Result<RequestBuilder, Error> {
        let request = self.client.get(url);
        Ok(RequestBuilder { request })
    }

    /// Construct a builder to POST to the given URL.
    fn post(&self, url: &str) -> Result<RequestBuilder, Error> {
        let request = self.client.post(url);
        Ok(RequestBuilder { request })
    }
}

/// Shorthand for generating a get request.
fn get(url: &str) -> Result<Response, Error> {
    Ok(Response {
        response: reqwest::blocking::get(url)?,
    })
}