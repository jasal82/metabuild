use rhai::EvalAltResult;

macro_rules! error {
    ($fmt:literal $($t:tt)*) => (
        Box::new(rhai::EvalAltResult::ErrorRuntime(
            format!($fmt $($t)*).into(),
            rhai::Position::NONE
        ))
    )
}

pub mod arch;
pub mod cmd;
pub mod fs;
pub mod git;
pub mod net;
pub mod re;
pub mod str;
pub mod sys;
pub mod toml;
pub mod yaml;

pub type RhaiResult<T> = std::result::Result<T, Box<EvalAltResult>>;