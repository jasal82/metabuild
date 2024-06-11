use koto::{derive::*, prelude::*, Result};
use std::cell::RefCell;
use std::process::{Command, Stdio};
use std::rc::Rc;

#[cfg(target_os = "windows")]
mod internal {
    use std::process::Command;

    fn quote(s: &str) -> String {
        if s.contains(' ') {
            format!(r#"'{s}'"#)
        } else {
            s.into()
        }
    }

    pub fn new_shell_command() -> Command {
        Command::new("powershell")
    }

    pub fn add_shell_arguments(command: &mut Command, args: &[String]) {
        // Append '; exit $LASTEXITCODE' to the command so that the exit
        // code of the actual command is returned instead of the exit code
        // of powershell. This is done to make the behavior of the command
        // module consistent across platforms. Linux shells already return
        // the exit code of the last command in the script without the need
        // for an explicit exit command.
        let mut args = args.to_owned();
        args.append(&mut vec![";".into(), "exit".into(), "$LASTEXITCODE".into()]);
        command.arg("-Command");
        command.arg(format!(
            "& {{{}}}",
            args.iter()
                .map(|a| quote(a))
                .collect::<Vec<String>>()
                .join(" ")
        ));
    }
}

#[cfg(target_os = "linux")]
mod internal {
    use std::process::Command;

    fn quote(s: &str) -> String {
        if s.contains(" ") {
            format!(r#""{}""#, s)
        } else {
            s.into()
        }
    }

    pub fn new_shell_command() -> Command {
        Command::new("/bin/sh")
    }

    pub fn add_shell_arguments(command: &mut Command, args: &Vec<String>) {
        command.arg("-c");
        command.arg(
            args.iter()
                .map(|a| quote(a))
                .collect::<Vec<String>>()
                .join(" "),
        );
    }
}

#[derive(Clone)]
enum Routing {
    Piped,
    Inherit,
    Null,
    Unspecified,
}

struct CmdInternal {
    args: Vec<String>,
    envs: Vec<(String, String)>,
    env_remove: Vec<String>,
    env_clear: bool,
    current_dir: Option<String>,
    shell: bool,
    stdout: Routing,
    stderr: Routing,
}

#[derive(Clone, KotoCopy, KotoType)]
struct Cmd(Rc<RefCell<CmdInternal>>);

#[koto_impl]
impl Cmd {
    pub fn new(cmd: &KString) -> Self {
        Self(Rc::new(RefCell::new(CmdInternal {
            args: vec![cmd.to_string()],
            envs: vec![],
            env_remove: vec![],
            env_clear: false,
            current_dir: None,
            shell: false,
            stdout: Routing::Unspecified,
            stderr: Routing::Unspecified,
        })))
    }

    #[koto_method]
    pub fn arg(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(text)] => {
                ctx.instance_mut()?.0.borrow_mut().args.push(text.to_string());
                ctx.instance_result()
            }
            unexpected => type_error_with_slice("a string", unexpected),
        }
    }

    #[koto_method]
    pub fn args(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::List(args)] => {
                for arg in args.data().iter() {
                    match arg {
                        KValue::Str(s) => {
                            ctx.instance_mut()?.0.borrow_mut().args.push(s.to_string());
                        }
                        actual => return type_error("string", actual)
                    }
                }
                ctx.instance_result()
            }
            unexpected => type_error_with_slice("(args: list)", unexpected),
        }
    }

    #[koto_method]
    pub fn current_dir(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(dir)] => {
                ctx.instance_mut()?.0.borrow_mut().current_dir = Some(dir.to_string());
                ctx.instance_result()
            }
            unexpected => type_error_with_slice("(dir: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn env(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(key), KValue::Str(value)] => {
                ctx.instance_mut()?.0.borrow_mut().envs.push((key.to_string(), value.to_string()));
                ctx.instance_result()
            }
            unexpected => type_error_with_slice("(key: string, value: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn envs(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Map(envs)] => {
                for (key, value) in envs.data().iter() {
                    match value {
                        KValue::Str(value) => {
                            ctx.instance_mut()?.0.borrow_mut().envs.push((key.to_string(), value.to_string()));
                        }
                        actual => return type_error("string", actual)
                    }
                }
                ctx.instance_result()
            }
            unexpected => type_error_with_slice("(envs: map)", unexpected),
        }
    }

    #[koto_method]
    pub fn env_clear(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [] => {
                ctx.instance_mut()?.0.borrow_mut().env_clear = true;
                ctx.instance_result()
            }
            unexpected => type_error_with_slice("()", unexpected),
        }
    }

    #[koto_method]
    pub fn env_remove(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(key)] => {
                ctx.instance_mut()?.0.borrow_mut().env_remove.push(key.to_string());
                ctx.instance_result()
            }
            unexpected => type_error_with_slice("(key: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn shell(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [] => {
                ctx.instance_mut()?.0.borrow_mut().shell = true;
                ctx.instance_result()
            }
            unexpected => type_error_with_slice("()", unexpected),
        }
    }

    #[koto_method]
    pub fn stdout(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(routing)] => {
                let routing = match routing.as_str() {
                    "piped" => Routing::Piped,
                    "inherit" => Routing::Inherit,
                    "null" => Routing::Null,
                    unexpected => {
                        return runtime_error!(format!(
                            "Expected one of 'piped', 'inherit', or 'null', found '{}'",
                            unexpected
                        ))
                    }
                };
                ctx.instance_mut()?.0.borrow_mut().stdout = routing;
                ctx.instance_result()
            }
            unexpected => type_error_with_slice("(routing: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn stderr(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(routing)] => {
                let routing = match routing.as_str() {
                    "piped" => Routing::Piped,
                    "inherit" => Routing::Inherit,
                    "null" => Routing::Null,
                    unexpected => {
                        return runtime_error!(format!(
                            "Expected one of 'piped', 'inherit', or 'null', found '{}'",
                            unexpected
                        ))
                    }
                };
                ctx.instance_mut()?.0.borrow_mut().stderr = routing;
                ctx.instance_result()
            }
            unexpected => type_error_with_slice("(routing: string)", unexpected),
        }
    }

    fn build_cmd(&mut self) -> Command {
        let mut cmd = match self.0.borrow().shell {
            true => {
                let mut cmd: Command = internal::new_shell_command();
                internal::add_shell_arguments(&mut cmd, &self.0.borrow().args);
                cmd
            }
            false => {
                let mut cmd = Command::new(&self.0.borrow().args[0]);
                cmd.args(self.0.borrow().args.iter().skip(1));
                cmd
            }
        };

        if self.0.borrow().env_clear {
            cmd.env_clear();
        }

        for key in self.0.borrow().env_remove.iter() {
            cmd.env_remove(key);
        }

        for (key, value) in self.0.borrow().envs.iter() {
            cmd.env(key, value);
        }

        if let Some(dir) = &self.0.borrow().current_dir {
            let absolute_dir = dunce::canonicalize(dir).unwrap();
            cmd.current_dir(absolute_dir);
        }

        match self.0.borrow().stdout {
            Routing::Piped => {
                cmd.stdout(Stdio::piped());
            }
            Routing::Inherit => {
                cmd.stdout(Stdio::inherit());
            }
            Routing::Null => {
                cmd.stdout(Stdio::null());
            }
            Routing::Unspecified => {}
        };

        match self.0.borrow().stderr {
            Routing::Piped => {
                cmd.stderr(Stdio::piped());
            }
            Routing::Inherit => {
                cmd.stderr(Stdio::inherit());
            }
            Routing::Null => {
                cmd.stderr(Stdio::null());
            }
            Routing::Unspecified => {}
        };

        cmd
    }

    #[koto_method]
    pub fn execute(&mut self) -> Result<KValue> {
        let mut cmd = self.build_cmd();
        let output = cmd
            .output()
            .map_err(|e| koto::runtime::Error::from(format!("Failed to execute command: {:?}", e)))?;
        let result = KMap::new();
        result.insert("status", KNumber::from(output.status.code().unwrap()));
        result.insert("stdout", KString::from(String::from_utf8(output.stdout).unwrap()));
        result.insert("stderr", KString::from(String::from_utf8(output.stderr).unwrap()));
        Ok(result.into())
    }
}

impl KotoObject for Cmd {
    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Cmd");
        Ok(())
    }
}

impl From<Cmd> for KValue {
    fn from(cmd: Cmd) -> Self {
        KObject::from(cmd).into()
    }
}

pub fn split(command: &KString) -> Result<KValue> {
    shell_words::split(command.as_str()).map_err(|e| koto::Error::from(e.to_string())).map(|args| {
        let args = args
            .into_iter()
            .map(|s| KValue::Str(s.into()))
            .collect::<Vec<KValue>>();
        KTuple::from(args).into()
    })
}

pub fn join(args: &KTuple) -> Result<KValue> {
    let args: Vec<&str> = args.iter().map(|e| match e {
        KValue::Str(s) => Ok(s.as_str()),
        actual => type_error("string", actual)
    }).collect::<Result<Vec<_>>>()?;
    Ok(shell_words::join(args).into())
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("cmd");
    result.add_fn("split", |ctx| match ctx.args() {
        [KValue::Str(command)] => split(command).into(),
        unexpected => type_error_with_slice("(command: string)", unexpected),
    });
    result.add_fn("join", |ctx| match ctx.args() {
        [KValue::Tuple(args)] => join(args).into(),
        unexpected => type_error_with_slice("(args: list)", unexpected),
    });
    result.add_fn("new", |ctx| match ctx.args() {
        [KValue::Str(command)] => Ok(Cmd::new(command).into()),
        unexpected => type_error_with_slice("(command: string)", unexpected),
    });

    result
}
