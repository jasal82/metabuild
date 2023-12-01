use rune::runtime::{Object, Value, VmResult};
use rune::{Any, ContextError, Module, vm_try};
use std::process::{Command, Stdio};

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

#[derive(Any, PartialEq, Eq)]
#[rune(item = ::cmd)]
enum Routing {
    #[rune(constructor)]
    Piped,
    #[rune(constructor)]
    Inherit,
    #[rune(constructor)]
    Null,
    #[rune(constructor)]
    Unspecified,
}

#[derive(Any)]
#[rune(item = ::cmd, name = Command)]
struct Cmd {
    args: Vec<String>,
    envs: Vec<(String, String)>,
    env_remove: Vec<String>,
    env_clear: bool,
    current_dir: Option<String>,
    shell: bool,
    stdout: Routing,
    stderr: Routing,
}

#[derive(Any)]
#[rune(item = ::cmd)]
struct Output {
    status: i64,
    stdout: String,
    stderr: String,
}

impl Output {
    #[rune::function]
    pub fn status(&self) -> i64 {
        self.status
    }

    #[rune::function]
    pub fn stdout(&self) -> String {
        self.stdout.clone()
    }

    #[rune::function]
    pub fn stderr(&self) -> String {
        self.stderr.clone()
    }
}

impl Cmd {
    #[rune::function(path = Self::new)]
    pub fn new(cmd: &str) -> Self {
        Self {
            args: vec![cmd.into()],
            envs: vec![],
            env_remove: vec![],
            env_clear: false,
            current_dir: None,
            shell: false,
            stdout: Routing::Unspecified,
            stderr: Routing::Unspecified,
        }
    }

    #[rune::function]
    pub fn arg(mut self, arg: &str) -> Cmd {
        self.args.push(arg.into());
        self
    }

    #[rune::function]
    pub fn args(mut self, args: &[Value]) -> VmResult<Cmd> {
        for arg in args {
            match arg {
                Value::String(s) => {
                    self.args.push(vm_try!(s.borrow_ref()).to_string());
                }
                actual => {
                    return VmResult::expected::<String>(vm_try!(actual.type_info()));
                }
            }
        }
        VmResult::Ok(self)
    }

    #[rune::function]
    pub fn current_dir(mut self, dir: &str) -> Cmd {
        self.current_dir = Some(dir.into());
        self
    }

    #[rune::function]
    pub fn env(mut self, key: &str, value: &str) -> Cmd {
        self.envs.push((key.into(), value.into()));
        self
    }

    #[rune::function]
    pub fn envs(mut self, envs: Object) -> Cmd {
        for (key, value) in envs {
            self.envs.push((
                key.to_string(),
                value.into_string().unwrap().take().unwrap().to_string(),
            ));
        }
        self
    }

    #[rune::function]
    pub fn env_clear(mut self) -> Cmd {
        self.env_clear = true;
        self
    }

    #[rune::function]
    pub fn env_remove(mut self, key: &str) -> Cmd {
        self.env_remove.push(key.into());
        self
    }

    #[rune::function]
    pub fn shell(mut self) -> Cmd {
        self.shell = true;
        self
    }

    #[rune::function]
    pub fn stdout(mut self, routing: Routing) -> Cmd {
        self.stdout = routing;
        self
    }

    #[rune::function]
    pub fn stderr(mut self, routing: Routing) -> Cmd {
        self.stderr = routing;
        self
    }

    fn build_cmd(&mut self) -> Command {
        let mut cmd = match self.shell {
            true => {
                let mut cmd: Command = internal::new_shell_command();
                internal::add_shell_arguments(&mut cmd, &self.args);
                cmd
            }
            false => {
                let mut cmd = Command::new(&self.args[0]);
                cmd.args(self.args.iter().skip(1));
                cmd
            }
        };

        if self.env_clear {
            cmd.env_clear();
        }

        for key in self.env_remove.iter() {
            cmd.env_remove(key);
        }

        for (key, value) in self.envs.iter() {
            cmd.env(key, value);
        }

        if let Some(dir) = &self.current_dir {
            let absolute_dir = dunce::canonicalize(dir).unwrap();
            cmd.current_dir(absolute_dir);
        }

        match self.stdout {
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

        match self.stderr {
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

    #[rune::function]
    pub fn execute(&mut self) -> Result<Output, anyhow::Error> {
        let mut cmd = self.build_cmd();
        let output = cmd
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute command: {:?}", self.args));
        Ok(Output {
            status: output.status.code().unwrap() as i64,
            stdout: String::from_utf8(output.stdout).unwrap(),
            stderr: String::from_utf8(output.stderr).unwrap(),
        })
    }
}

#[rune::function]
pub fn split(command: &str) -> Result<Vec<String>, anyhow::Error> {
    shell_words::split(command).map_err(|e| e.into())
}

#[rune::function]
pub fn join(args: Vec<String>) -> String {
    shell_words::join(args)
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("cmd")?;
    module.function_meta(split)?;
    module.function_meta(join)?;
    module.ty::<Cmd>()?;
    module.ty::<Routing>()?;
    module.ty::<Output>()?;

    module.function_meta(Output::status)?;
    module.function_meta(Output::stdout)?;
    module.function_meta(Output::stderr)?;

    module.function_meta(Cmd::new)?;
    module.function_meta(Cmd::arg)?;
    module.function_meta(Cmd::args)?;
    module.function_meta(Cmd::current_dir)?;
    module.function_meta(Cmd::env)?;
    module.function_meta(Cmd::envs)?;
    module.function_meta(Cmd::env_clear)?;
    module.function_meta(Cmd::env_remove)?;
    module.function_meta(Cmd::shell)?;
    module.function_meta(Cmd::stdout)?;
    module.function_meta(Cmd::stderr)?;
    module.function_meta(Cmd::execute)?;

    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd() {
        
    }
}
