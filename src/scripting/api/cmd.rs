use rune::{Any, ContextError, Module};
use rune::runtime::Object;
use std::process::{Command, Stdio};

#[cfg(target_os = "windows")]
mod internal {
    use std::process::Command;

    fn quote(s: &str) -> String {
        if s.contains(" ") {
            format!(r#"'{}'"#, s)
        } else {
            s.into()
        }
    }

    pub fn new_shell_command() -> Command {
        Command::new("powershell")
    }

    pub fn add_shell_arguments(command: &mut Command, args: &Vec<String>) {
        command.arg("-Command");
        command.arg(format!("& {{{}}}", args.iter().map(|a| quote(a)).collect::<Vec<String>>().join(" ")));
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
        command.arg(args.iter().map(|a| quote(a)).collect::<Vec<String>>().join(" "));
    }
}

enum PipeRouting {
    Piped,
    Inherit,
    Null,
    Unspecified,
}

#[derive(Any)]
struct Cmd {
    args: Vec<String>,
    envs: Vec<(String, String)>,
    env_remove: Vec<String>,
    env_clear: bool,
    current_dir: Option<String>,
    shell: bool,
    stdout: PipeRouting,
    stderr: PipeRouting,
}

impl Cmd {
    pub fn new(cmd: &str) -> Self {
        Self {
            args: vec![cmd.into()],
            envs: vec![],
            env_remove: vec![],
            env_clear: false,
            current_dir: None,
            shell: false,
            stdout: PipeRouting::Unspecified,
            stderr: PipeRouting::Unspecified,
        }
    }

    pub fn arg(&mut self, arg: &str) {
        self.args.push(arg.into());
    }

    pub fn args(&mut self, args: Vec<String>) {
        for arg in args {
            self.args.push(arg);
        }
    }

    pub fn current_dir(&mut self, dir: &str) {
        self.current_dir = Some(dir.into());
    }

    pub fn env(&mut self, key: &str, value: &str) {
        self.envs.push((key.into(), value.into()));
    }

    pub fn envs(&mut self, envs: Object) {
        for (key, value) in envs {
            self.envs.push((key.to_string(), value.into_string().unwrap().take().unwrap()));
        }
    }

    pub fn env_clear(&mut self) {
        self.env_clear = true;
    }

    pub fn env_remove(&mut self, key: &str) {
        self.env_remove.push(key.into());
    }

    pub fn shell(&mut self) {
        self.shell = true;
    }

    pub fn stdout(&mut self, stdout: &str) {
        match stdout {
            "pipe" => self.stdout = PipeRouting::Piped,
            "inherit" => self.stdout = PipeRouting::Inherit,
            "null" => self.stdout = PipeRouting::Null,
            "unspecified" => self.stdout = PipeRouting::Unspecified,
            _ => panic!("Invalid stdout routing: {}", stdout),
        }
    }

    pub fn stderr(&mut self, stderr: &str) {
        match stderr {
            "pipe" => self.stderr = PipeRouting::Piped,
            "inherit" => self.stderr = PipeRouting::Inherit,
            "null" => self.stderr = PipeRouting::Null,
            "unspecified" => self.stderr = PipeRouting::Unspecified,
            _ => panic!("Invalid stderr routing: {}", stderr),
        }
    }

    fn build_cmd(&mut self) -> Command {
        let mut cmd = match self.shell {
            true => {
                let mut cmd = internal::new_shell_command();
                internal::add_shell_arguments(&mut cmd, &self.args);
                cmd
            },
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
            PipeRouting::Piped => { cmd.stdout(Stdio::piped()); },
            PipeRouting::Inherit => { cmd.stdout(Stdio::inherit()); },
            PipeRouting::Null => { cmd.stdout(Stdio::null()); },
            PipeRouting::Unspecified => {},
        };

        match self.stderr {
            PipeRouting::Piped => { cmd.stderr(Stdio::piped()); },
            PipeRouting::Inherit => { cmd.stderr(Stdio::inherit()); },
            PipeRouting::Null => { cmd.stderr(Stdio::null()); },
            PipeRouting::Unspecified => {},
        };

        cmd
    }

    pub fn execute(&mut self) -> i64 {
        let mut cmd = self.build_cmd();
        cmd.status().expect(&format!("Failed to execute command: {:?}", self.args)).code().unwrap_or(1) as i64
    }

    pub fn output(&mut self) -> String {
        let mut cmd = self.build_cmd();
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        let output = cmd.output().expect(&format!("Failed to execute command: {:?}", self.args));
        String::from_utf8(output.stdout).unwrap()
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("cmd");
    module.ty::<Cmd>()?;
    module.function(["Command", "new"], Cmd::new)?;
    module.inst_fn("arg", Cmd::arg)?;
    module.inst_fn("args", Cmd::args)?;
    module.inst_fn("current_dir", Cmd::current_dir)?;
    module.inst_fn("env", Cmd::env)?;
    module.inst_fn("envs", Cmd::envs)?;
    module.inst_fn("env_clear", Cmd::env_clear)?;
    module.inst_fn("env_remove", Cmd::env_remove)?;
    module.inst_fn("shell", Cmd::shell)?;
    module.inst_fn("stdout", Cmd::stdout)?;
    module.inst_fn("stderr", Cmd::stderr)?;
    module.inst_fn("execute", Cmd::execute)?;
    module.inst_fn("output", Cmd::output)?;
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd() {
        let mut cmd = Cmd::new("echo");
        cmd.arg("Hello World");
        cmd.shell();
        #[cfg(target_os = "windows")]
        assert_eq!(cmd.output(), "Hello World\r\n");
        #[cfg(target_os = "linux")]
        assert_eq!(cmd.output(), "Hello World\n");
    }
}