use koto::prelude::*;
use koto::runtime::Result;
use std::sync::Arc;

pub fn make_module() -> KMap {
    let result = KMap::with_type("http");
    result.add_fn("client", |_| Ok(Client::new().into()));
    result
}

#[derive(Clone, Debug)]
pub struct Client {
    client: ureq::Agent,
}

#[derive(Clone, Debug)]
pub struct Request {
    request: Option<ureq::Request>,
}

#[derive(Debug)]
pub struct Response {
    response: Option<ureq::Response>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            client: ureq::builder()
                .tls_config(Arc::new(crate::TLS_CONFIG.clone()))
                .build(),
        }
    }

    pub fn get(&self, url: &str) -> Request {
        Request {
            request: Some(self.client.get(url)),
        }
    }

    pub fn put(&self, url: &str) -> Request {
        Request {
            request: Some(self.client.put(url)),
        }
    }

    pub fn post(&self, url: &str) -> Request {
        Request {
            request: Some(self.client.post(url)),
        }
    }

    pub fn delete(&self, url: &str) -> Request {
        Request {
            request: Some(self.client.delete(url)),
        }
    }
}

impl KotoType for Client {
    const TYPE: &'static str = "Client";
}

impl KotoObject for Client {
    fn object_type(&self) -> KString {
        CLIENT_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        KObject::from(Client {
            client: self.client.clone(),
        })
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        CLIENT_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Client");
        Ok(())
    }
}

impl From<Client> for Value {
    fn from(client: Client) -> Self {
        KObject::from(client).into()
    }
}

fn make_client_entries() -> ValueMap {
    ObjectEntryBuilder::<Client>::new()
        .method("get", |ctx| match ctx.args {
            [Value::Str(url)] => Ok(ctx.instance()?.get(url).into()),
            unexpected => type_error_with_slice("(url: string)", unexpected),
        })
        .method("put", |ctx| match ctx.args {
            [Value::Str(url)] => Ok(ctx.instance()?.put(url).into()),
            unexpected => type_error_with_slice("(url: string)", unexpected),
        })
        .method("post", |ctx| match ctx.args {
            [Value::Str(url)] => Ok(ctx.instance()?.post(url).into()),
            unexpected => type_error_with_slice("(url: string)", unexpected),
        })
        .method("delete", |ctx| match ctx.args {
            [Value::Str(url)] => Ok(ctx.instance()?.delete(url).into()),
            unexpected => type_error_with_slice("(url: string)", unexpected),
        })
        .build()
}

thread_local! {
    static CLIENT_TYPE_STRING: KString = Client::TYPE.into();
    static CLIENT_ENTRIES: ValueMap = make_client_entries();
}

impl Request {
    pub fn call(&mut self) -> Result<Response> {
        Ok(Response {
            response: Some(
                self.request
                    .take()
                    .expect("Invalid request instance")
                    .call()
                    .map_err(|e: ureq::Error| koto::runtime::RuntimeError::from(e.to_string()))?,
            ),
        })
    }

    pub fn set(&mut self, key: &str, value: &str) -> Self {
        Self {
            request: Some(
                self.request
                    .take()
                    .expect("Invalid request instance")
                    .set(key, value),
            ),
        }
    }

    pub fn send_string(&mut self, body: &str) -> Result<Response> {
        self.request
            .take()
            .expect("Invalid request instance")
            .send_string(body)
            .map(|r| Response { response: Some(r) })
            .map_err(|e: ureq::Error| koto::runtime::RuntimeError::from(e.to_string()))
    }
}

impl KotoType for Request {
    const TYPE: &'static str = "Request";
}

impl KotoObject for Request {
    fn object_type(&self) -> KString {
        REQUEST_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        KObject::from(Request {
            request: self.request.clone(),
        })
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        REQUEST_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Request");
        Ok(())
    }
}

impl From<Request> for Value {
    fn from(request: Request) -> Self {
        KObject::from(request).into()
    }
}

fn make_request_entries() -> ValueMap {
    ObjectEntryBuilder::<Request>::new()
        .method("call", |ctx| match ctx.args {
            [] => Ok(ctx.instance_mut()?.call()?.into()),
            unexpected => type_error_with_slice("()", unexpected),
        })
        .method("set", |ctx| match ctx.args {
            [Value::Str(key), Value::Str(value)] => Ok(ctx.instance_mut()?.set(key, value).into()),
            unexpected => type_error_with_slice("(key: string, value: string)", unexpected),
        })
        .method("send_string", |ctx| match ctx.args {
            [Value::Str(body)] => Ok(ctx.instance_mut()?.send_string(body)?.into()),
            unexpected => type_error_with_slice("(body: string)", unexpected),
        })
        .build()
}

thread_local! {
    static REQUEST_TYPE_STRING: KString = Request::TYPE.into();
    static REQUEST_ENTRIES: ValueMap = make_request_entries();
}

impl Response {
    pub fn status(&self) -> u16 {
        self.response.as_ref().unwrap().status()
    }

    pub fn status_text(&self) -> String {
        self.response.as_ref().unwrap().status_text().to_string()
    }

    pub fn into_string(&mut self) -> Result<String> {
        self.response
            .take()
            .expect("Invalid response instance")
            .into_string()
            .map_err(|e: std::io::Error| koto::runtime::RuntimeError::from(e.to_string()))
    }

    pub fn into_file(&mut self, file: &str) -> Result<Value> {
        let mut file = std::fs::File::create(file)
            .map_err(|e| koto::runtime::RuntimeError::from(e.to_string()))?;
        std::io::copy(
            &mut self
                .response
                .take()
                .expect("Invalid response instance")
                .into_reader(),
            &mut file,
        )
        .map_err(|e| koto::runtime::RuntimeError::from(e.to_string()))?;
        Ok(Value::Null)
    }
}

impl KotoType for Response {
    const TYPE: &'static str = "Response";
}

impl KotoObject for Response {
    fn object_type(&self) -> KString {
        RESPONSE_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        Self { response: None }.into()
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        RESPONSE_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append(format!(
            "Response: {}",
            self.response.as_ref().unwrap().status()
        ));
        Ok(())
    }
}

impl From<Response> for Value {
    fn from(response: Response) -> Self {
        KObject::from(response).into()
    }
}

fn make_response_entries() -> ValueMap {
    ObjectEntryBuilder::<Response>::new()
        .method("status", |ctx| match ctx.args {
            [] => Ok(ctx.instance()?.status().into()),
            unexpected => type_error_with_slice("()", unexpected),
        })
        .method("status_text", |ctx| match ctx.args {
            [] => Ok(ctx.instance()?.status_text().into()),
            unexpected => type_error_with_slice("()", unexpected),
        })
        .method("into_string", |ctx| match ctx.args {
            [] => Ok(ctx.instance_mut()?.into_string()?.into()),
            unexpected => type_error_with_slice("()", unexpected),
        })
        .method("into_file", |ctx| match ctx.args {
            [Value::Str(file)] => Ok(ctx.instance_mut()?.into_file(file)?.into()),
            unexpected => type_error_with_slice("(file: string)", unexpected),
        })
        .build()
}

thread_local! {
    static RESPONSE_TYPE_STRING: KString = Response::TYPE.into();
    static RESPONSE_ENTRIES: ValueMap = make_response_entries();
}
