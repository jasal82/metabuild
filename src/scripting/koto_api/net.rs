use crate::net;
use koto::prelude::*;
use koto::runtime::Result;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug)]
struct Client {
    headers: HashMap<String, String>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    pub fn header(&mut self, key: &str, value: &str) -> Result<Value> {
        self.headers.insert(key.to_string(), value.to_string());
        Ok(Value::Null)
    }

    pub fn download_file(&self, url: &str, file: &str) -> Result<Value> {
        net::download_file(url, Path::new(file), &self.headers)
            .map(|_| Value::Null)
            .map_err(|e| koto::prelude::RuntimeError::from(e.to_string()))
    }

    pub fn upload_file(&self, url: &str, file: &str) -> Result<Value> {
        net::upload_file(url, Path::new(file), &self.headers)
            .map(|_| Value::Null)
            .map_err(|e| koto::prelude::RuntimeError::from(e.to_string()))
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
            headers: self.headers.clone(),
        })
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        CLIENT_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
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
        .method("header", |ctx| match ctx.args {
            [Value::Str(key), Value::Str(value)] => ctx.instance_mut()?.header(key, value),
            unexpected => type_error_with_slice("(key: string, value: string)", unexpected),
        })
        .method("download_file", |ctx| match ctx.args {
            [Value::Str(url), Value::Str(file)] => ctx.instance()?.download_file(url, file),
            unexpected => type_error_with_slice("(url: string, file: string)", unexpected),
        })
        .method("upload_file", |ctx| match ctx.args {
            [Value::Str(url), Value::Str(file)] => ctx.instance()?.upload_file(url, file),
            unexpected => type_error_with_slice("(url: string, file: string)", unexpected),
        })
        .build()
}

thread_local! {
    static CLIENT_TYPE_STRING: KString = Client::TYPE.into();
    static CLIENT_ENTRIES: ValueMap = make_client_entries();
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("net");
    result.add_fn("client", |ctx| Ok(Client::new().into()));
    result
}
