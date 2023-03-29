use rune::{ContextError, Module};
use rune::runtime::{Object, Value, Vec};
use toml::Table;

fn convert_toml_to_rune(value: toml::Value) -> Value {
    match value {
        toml::Value::String(s) => s.into(),
        toml::Value::Integer(i) => i.into(),
        toml::Value::Float(f) => f.into(),
        toml::Value::Boolean(b) => b.into(),
        toml::Value::Datetime(d) => d.to_string().into(),
        toml::Value::Array(a) => convert_array_to_vector(a),
        toml::Value::Table(t) => convert_table_to_object(t).into(),
    }
}

fn convert_array_to_vector(array: std::vec::Vec<toml::Value>) -> Value {
    let mut vec = Vec::new();
    for e in array {
        vec.push(convert_toml_to_rune(e));
    }
    vec.into()
}

fn convert_table_to_object(table: Table) -> Object {
    let mut map = Object::new();
    for (k, v) in table {
        map.insert(k.into(), convert_toml_to_rune(v));
    }
    map
}

pub fn from_file(file: &str) -> Result<Object, anyhow::Error> {
    Ok(from_string(&std::fs::read_to_string(file)?)?)
}

pub fn from_string(content: &str) -> Result<Object, anyhow::Error> {
    Ok(convert_table_to_object(content.parse::<Table>()?))
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("toml");
    module.function(["from_file"], from_file)?;
    module.function(["from_string"], from_string)?;
    Ok(module)
}