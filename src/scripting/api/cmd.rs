use koto::prelude::*;
use koto::runtime::Result;
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

#[derive(Clone)]
struct Cmd(Rc<RefCell<CmdInternal>>);

impl Cmd {
    pub fn new(cmd: &str) -> Self {
        Self(Rc::new(RefCell::new(CmdInternal {
            args: vec![cmd.into()],
            envs: vec![],
            env_remove: vec![],
            env_clear: false,
            current_dir: None,
            shell: false,
            stdout: Routing::Unspecified,
            stderr: Routing::Unspecified,
        })))
    }

    pub fn arg(&self, arg: &str) -> Result<Cmd> {
        self.0.borrow_mut().args.push(arg.into());
        Ok(self.clone())
    }

    pub fn args(&self, args: &[Value]) -> Result<Cmd> {
        for arg in args {
            match arg {
                Value::Str(s) => {
                    self.0.borrow_mut().args.push(s.to_string());
                }
                actual => return type_error("string", actual)
            }
        }
        Ok(self.clone())
    }

    pub fn current_dir(&self, dir: &str) -> Result<Cmd> {
        self.0.borrow_mut().current_dir = Some(dir.into());
        Ok(self.clone())
    }

    pub fn env(&mut self, key: &str, value: &str) -> Result<Cmd> {
        self.0.borrow_mut().envs.push((key.into(), value.into()));
        Ok(self.clone())
    }

    pub fn envs(&mut self, envs: KMap) -> Result<Cmd> {
        for (key, value) in envs.data().iter() {
            match value {
                Value::Str(value) => {
                    self.0.borrow_mut().envs.push((key.to_string(), value.to_string()));
                }
                actual => return type_error("string", actual)
            }
        }
        Ok(self.clone())
    }

    pub fn env_clear(&mut self) -> Result<Cmd> {
        self.0.borrow_mut().env_clear = true;
        Ok(self.clone())
    }

    pub fn env_remove(&mut self, key: &str) -> Result<Cmd> {
        self.0.borrow_mut().env_remove.push(key.into());
        Ok(self.clone())
    }

    pub fn shell(&mut self) -> Result<Cmd> {
        self.0.borrow_mut().shell = true;
        Ok(self.clone())
    }

    pub fn stdout(&mut self, routing: Routing) -> Result<Cmd> {
        self.0.borrow_mut().stdout = routing;
        Ok(self.clone())
    }

    pub fn stderr(&mut self, routing: Routing) -> Result<Cmd> {
        self.0.borrow_mut().stderr = routing;
        Ok(self.clone())
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

    pub fn execute(&mut self) -> Result<(i64, String, String)> {
        let mut cmd = self.build_cmd();
        let output = cmd
            .output()
            .map_err(|e| make_runtime_error!(format!("Failed to execute command: {:?}", e)))?;
        Ok((output.status.code().unwrap() as i64,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        ))
    }
}

impl KotoType for Cmd {
    const TYPE: &'static str = "Cmd";
}

impl KotoObject for Cmd {
    fn object_type(&self) -> KString {
        CMD_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        self.clone().into()
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        CMD_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Cmd");
        Ok(())
    }
}

impl From<Cmd> for Value {
    fn from(cmd: Cmd) -> Self {
        KObject::from(cmd).into()
    }
}

fn make_cmd_entries() -> ValueMap {
    ObjectEntryBuilder::<Cmd>::new()
        .method("arg", |ctx| match ctx.args {
            [Value::Str(text)] => {
                ctx.instance_mut()?.arg(text).map(|v| v.into())
            }
            unexpected => type_error_with_slice("a string", unexpected),
        })
        .method("args", |ctx| match ctx.args {
            [Value::List(args)] => {
                ctx.instance_mut()?.args(args.data().as_slice()).map(|v| v.into())
            }
            unexpected => type_error_with_slice("(args: list)", unexpected),
        })
        .method("current_dir", |ctx| match ctx.args {
            [Value::Str(dir)] => {
                ctx.instance_mut()?.current_dir(dir).map(|v| v.into())
            }
            unexpected => type_error_with_slice("(dir: string)", unexpected),
        })
        .method("env", |ctx| match ctx.args {
            [Value::Str(key), Value::Str(value)] => {
                ctx.instance_mut()?.env(key, value).map(|v| v.into())
            }
            unexpected => type_error_with_slice("(key: string, value: string)", unexpected),
        })
        .method("envs", |ctx| match ctx.args {
            [Value::Map(envs)] => {
                ctx.instance_mut()?.envs(envs.clone()).map(|v| v.into())
            }
            unexpected => type_error_with_slice("(envs: map)", unexpected),
        })
        .method("env_clear", |ctx| match ctx.args {
            [] => {
                ctx.instance_mut()?.env_clear().map(|v| v.into())
            }
            unexpected => type_error_with_slice("()", unexpected),
        })
        .method("env_remove", |ctx| match ctx.args {
            [Value::Str(key)] => {
                ctx.instance_mut()?.env_remove(key).map(|v| v.into())
            }
            unexpected => type_error_with_slice("(key: string)", unexpected),
        })
        .method("shell", |ctx| match ctx.args {
            [] => {
                ctx.instance_mut()?.shell().map(|v| v.into())
            }
            unexpected => type_error_with_slice("()", unexpected),
        })
        .method("stdout", |ctx| match ctx.args {
            [Value::Str(routing)] => {
                let routing = match routing.as_str() {
                    "piped" => Routing::Piped,
                    "inherit" => Routing::Inherit,
                    "null" => Routing::Null,
                    unexpected => {
                        return Err(make_runtime_error!(format!(
                            "Expected one of 'piped', 'inherit', or 'null', found '{}'",
                            unexpected
                        )))
                    }
                };
                ctx.instance_mut()?.stdout(routing).map(|v| v.into())
            }
            unexpected => type_error_with_slice("(routing: string)", unexpected),
        })
        .method("stderr", |ctx| match ctx.args {
            [Value::Str(routing)] => {
                let routing = match routing.as_str() {
                    "piped" => Routing::Piped,
                    "inherit" => Routing::Inherit,
                    "null" => Routing::Null,
                    unexpected => {
                        return Err(make_runtime_error!(format!(
                            "Expected one of 'piped', 'inherit', or 'null', found '{}'",
                            unexpected
                        )))
                    }
                };
                ctx.instance_mut()?.stderr(routing).map(|v| v.into())
            }
            unexpected => type_error_with_slice("(routing: string)", unexpected),
        })
        .method("execute", |ctx| match ctx.args {
            [] => {
                let output = ctx.instance_mut()?.execute()?;
                let result = KMap::new();
                result.add_value("status", output.0.into());
                result.add_value("stdout", output.1.into());
                result.add_value("stderr", output.2.into());
                Ok(result.into())
            }
            unexpected => type_error_with_slice("()", unexpected),
        })

        .build()
}

thread_local! {
    static CMD_TYPE_STRING: KString = Cmd::TYPE.into();
    static CMD_ENTRIES: ValueMap = make_cmd_entries();
}


pub fn split(command: &str) -> Result<Value> {
    shell_words::split(command).map_err(|e| make_runtime_error!(e.to_string())).map(|args| {
        let args = args
            .into_iter()
            .map(|s| Value::Str(s.into()))
            .collect::<Vec<Value>>();
        KTuple::from(args).into()
    })
}

pub fn join(args: Vec<String>) -> String {
    shell_words::join(args)
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("cmd");
    result.add_fn("split", |ctx| match ctx.args() {
        [Value::Str(command)] => split(command).into(),
        unexpected => type_error_with_slice("(command: string)", unexpected),
    });
    result.add_fn("join", |ctx| match ctx.args() {
        [Value::Tuple(args)] => {
            let args = args.into_iter()
                .map(|a| match a {
                    Value::Str(s) => Ok(s.to_string()),
                    actual => type_error("string", actual),
                })
                .collect::<Result<Vec<String>>>()?;
            Ok(join(args).into())
        }
        unexpected => type_error_with_slice("(args: list)", unexpected),
    });
    result.add_fn("new", |ctx| match ctx.args() {
        [Value::Str(command)] => Ok(Cmd::new(command).into()),
        unexpected => type_error_with_slice("(command: string)", unexpected),
    });

    result
}
