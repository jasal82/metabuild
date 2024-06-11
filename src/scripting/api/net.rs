use crate::net;
use koto::{derive::*, prelude::*, Result};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, KotoCopy, KotoType)]
struct Client {
    headers: HashMap<String, String>,
}

#[koto_impl]
impl Client {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    #[koto_method]
    pub fn header(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(key), KValue::Str(value)] => {
                ctx.instance_mut()?.headers.insert(key.to_string(), value.to_string());
                Ok(KValue::Null)
            },
            unexpected => type_error_with_slice("(key: string, value: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn download_file(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(url), KValue::Str(file)] => {
                net::download_file(url, Path::new(file.as_str()), &ctx.instance()?.headers)
                    .map(|_| KValue::Null)
                    .map_err(|e| koto::runtime::Error::from(e.to_string()))
            },
            unexpected => type_error_with_slice("(url: string, file: string)", unexpected),
        }

    }

    #[koto_method]
    pub fn upload_file(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(url), KValue::Str(file)] => {
                net::upload_file(url, Path::new(file.as_str()), &ctx.instance()?.headers)
                    .map(|_| KValue::Null)
                    .map_err(|e| koto::runtime::Error::from(e.to_string()))
            },
            unexpected => type_error_with_slice("(url: string, file: string)", unexpected),
        }
    }
}

impl KotoObject for Client {
    fn display(&self, _: &mut DisplayContext) -> Result<()> {
        Ok(())
    }
}

impl From<Client> for KValue {
    fn from(client: Client) -> Self {
        KObject::from(client).into()
    }
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("net");
    result.add_fn("client", |_| Ok(Client::new().into()));
    result
}
