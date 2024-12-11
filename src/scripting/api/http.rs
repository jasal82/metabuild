use koto::{derive::*, prelude::*, Result};
use std::rc::Rc;
use std::cell::RefCell;

pub fn make_module() -> KMap {
    let result = KMap::with_type("http");
    result.add_fn("client", |_| Ok(Client::new().into()));
    result
}

#[derive(Clone, Debug, KotoCopy, KotoType)]
pub struct Client {
    client: ureq::Agent,
}

#[derive(Clone, Debug, KotoCopy, KotoType)]
pub struct Request {
    request: Option<ureq::Request>,
}

#[derive(Clone, Debug, KotoCopy, KotoType)]
pub struct Response {
    response: Rc<RefCell<Option<ureq::Response>>>,
}

#[koto_impl]
impl Client {
    pub fn new() -> Self {
        Self {
            client: ureq::builder()
                .build(),
        }
    }

    #[koto_method]
    pub fn get(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(url)] => {
                Ok(Request {
                    request: Some(ctx.instance()?.client.get(url)),
                }.into())
            },
            unexpected => type_error_with_slice("(url: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn put(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(url)] => {
                Ok(Request {
                    request: Some(ctx.instance()?.client.put(url)),
                }.into())
            },
            unexpected => type_error_with_slice("(url: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn post(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(url)] => {
                Ok(Request {
                    request: Some(ctx.instance()?.client.post(url)),
                }.into())
            },
            unexpected => type_error_with_slice("(url: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn delete(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(url)] => {
                Ok(Request {
                    request: Some(ctx.instance()?.client.delete(url)),
                }.into())
            },
            unexpected => type_error_with_slice("(url: string)", unexpected),
        }
    }
}

impl KotoObject for Client {
    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Client");
        Ok(())
    }
}

impl From<Client> for KValue {
    fn from(client: Client) -> Self {
        KObject::from(client).into()
    }
}

#[koto_impl]
impl Request {
    #[koto_method]
    pub fn call(&mut self) -> Result<KValue> {
        Ok(Response {
            response: Rc::new(RefCell::new(Some(
                self.request
                    .take()
                    .expect("Invalid request object")
                    .call()
                    .map_err(|e: ureq::Error| koto::runtime::Error::from(e.to_string()))?,
            ))),
        }.into())
    }

    #[koto_method]
    pub fn set(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(key), KValue::Str(value)] => {
                let r = ctx.instance_mut()?.request.take().unwrap();
                ctx.instance_mut()?.request = Some(r.set(key, value));
                ctx.instance_result()
            },
            unexpected => type_error_with_slice("(key: string, value: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn send_string(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(body)] => {
                ctx.instance_mut()?.request.take().unwrap().send_string(body)
                    .map(|r| Response { response: Rc::new(RefCell::new(Some(r))) }.into())
                    .map_err(|e: ureq::Error| koto::runtime::Error::from(e.to_string())).into()
            },
            unexpected => type_error_with_slice("(body: string)", unexpected),
        }
    }
}

impl KotoObject for Request {
    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Request");
        Ok(())
    }
}

impl From<Request> for KValue {
    fn from(request: Request) -> Self {
        KObject::from(request).into()
    }
}

#[koto_impl]
impl Response {
    #[koto_method]
    pub fn status(&self) -> Result<KValue> {
        Ok(self.response.borrow().as_ref().unwrap().status().into())
    }

    #[koto_method]
    pub fn status_text(&self) -> Result<KValue> {
        Ok(self.response.borrow().as_ref().unwrap().status_text().into())
    }

    #[koto_method]
    pub fn into_string(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [] => {
                ctx.instance_mut()?
                    .response
                    .borrow_mut()
                    .take()
                    .unwrap()
                    .into_string()
                    .map(|v| v.into())
                    .map_err(|e: std::io::Error| koto::runtime::Error::from(e.to_string())).into()
            },
            unexpected => type_error_with_slice("()", unexpected),
        }
    }

    #[koto_method]
    pub fn into_file(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(file)] => {
                let mut file = std::fs::File::create(file.as_str())
                    .map_err(|e| koto::runtime::Error::from(e.to_string()))?;
                std::io::copy(
                    &mut ctx.instance_mut()?
                        .response
                        .take()
                        .expect("Invalid response instance")
                        .into_reader(),
                    &mut file,
                )
                    .map_err(|e| koto::runtime::Error::from(e.to_string()))?;
                Ok(KValue::Null)
            },
            unexpected => type_error_with_slice("(string)", unexpected),
        }
    }
}

impl KotoObject for Response {
    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append(format!(
            "Response: {}",
            self.response.borrow().as_ref().unwrap().status()
        ));
        Ok(())
    }
}

impl From<Response> for KValue {
    fn from(response: Response) -> Self {
        KObject::from(response).into()
    }
}
