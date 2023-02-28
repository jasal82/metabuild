use rhai::Engine;
use super::RhaiResult;
use std::process::{Command, Stdio};

#[cfg(windows)]
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
#[cfg(linux)]
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

#[derive(Clone)]
struct Cmd {
    args: Vec<String>,
    envs: Vec<(String, String)>,
    env_remove: Vec<String>,
    env_clear: bool,
    current_dir: Option<String>,
    shell: bool,
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
        }
    }

    pub fn from_array(args: rhai::Array) -> RhaiResult<Self> {
        let mut str_args : Vec<String> = Vec::new();
        for arg in args {
            str_args.push(arg.into_string().unwrap());
        }

        Ok(Self {
            args: str_args,
            envs: vec![],
            env_remove: vec![],
            env_clear: false,
            current_dir: None,
            shell: false,
        })
    }

    pub fn arg(&mut self, arg: &str) {
        self.args.push(arg.into());
    }

    pub fn args(&mut self, args: rhai::Array) -> RhaiResult<()> {
        for arg in args {
            self.args.push(arg.into_string().unwrap());
        }

        Ok(())
    }

    pub fn current_dir(&mut self, dir: &str) {
        self.current_dir = Some(dir.into());
    }

    pub fn env(&mut self, key: &str, value: &str) {
        self.envs.push((key.into(), value.into()));
    }

    pub fn envs(&mut self, envs: rhai::Map) -> RhaiResult<()> {
        for (key, value) in envs {
            self.envs.push((key.to_string(), value.into_string().unwrap()));
        }

        Ok(())
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

    fn build_cmd(&mut self) -> RhaiResult<Command> {
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

        Ok(cmd)
    }

    pub fn execute(&mut self) -> RhaiResult<u8> {
        let mut cmd = self.build_cmd()?;
        Ok(cmd.status().expect(&format!("Failed to execute command: {:?}", self.args)).code().unwrap_or(1) as u8)
    }

    pub fn output(&mut self) -> RhaiResult<String> {
        let mut cmd = self.build_cmd()?;
        cmd.stdin(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        let output = cmd.output().expect(&format!("Failed to execute command: {:?}", self.args));
        Ok(String::from_utf8(output.stdout).unwrap())
    }
}

pub fn register(engine: &mut Engine) {
    engine.register_fn("cmd", Cmd::new);
    engine.register_fn("cmd", Cmd::from_array);
    engine.register_fn("arg", Cmd::arg);
    engine.register_fn("args", Cmd::args);
    engine.register_fn("current_dir", Cmd::current_dir);
    engine.register_fn("env", Cmd::env);
    engine.register_fn("envs", Cmd::envs);
    engine.register_fn("env_clear", Cmd::env_clear);
    engine.register_fn("env_remove", Cmd::env_remove);
    engine.register_fn("shell", Cmd::shell);
    engine.register_fn("execute", Cmd::execute);
    engine.register_fn("output", Cmd::output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd() {
        let mut cmd = Cmd::new("echo");
        cmd.arg("Hello World");
        cmd.shell();
        #[cfg(windows)]
        assert_eq!(cmd.output().unwrap(), "Hello World\r\n");
        #[cfg(linux)]
        assert_eq!(cmd.output().unwrap(), "Hello World\n");
    }
}