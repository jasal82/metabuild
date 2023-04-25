use rune::{Any, ContextError, Module};
use subprocess::{Popen, PopenConfig, Redirection};

const NEWLINE: u8 = 10;

#[derive(Any)]
struct Docker {
    process: Popen,
}

impl Docker {
    pub fn new(image: &str) -> Result<Docker, anyhow::Error> {
        let mut process = Popen::create(
            &["docker", "run", "-i", "--rm", image],
            PopenConfig {
                stdin: Redirection::Pipe,
                stdout: Redirection::Pipe,
                stderr: Redirection::Merge,
                ..Default::default()
            },
        )?;

        Ok(Docker { process })
    }

    pub fn send(&mut self, data: &str) -> std::io::Result<()> {
        self.stdin.write_all(data.as_bytes())?;
        self.stdin.write_all(&[NEWLINE])
    }
    
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("docker");
    module.ty::<Docker>()?;
    module.function(["Docker", "new"], Docker::new)?;
    module.inst_fn("send", Docker::send)?;
    Ok(module)
}