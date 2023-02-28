use rhai::{Engine, Dynamic, Module};
use super::RhaiResult;
use toml::Table;

fn convert_value_to_dynamic(value: toml::Value) -> Dynamic {
    match value {
        toml::Value::String(s) => s.into(),
        toml::Value::Integer(i) => i.into(),
        toml::Value::Float(f) => f.into(),
        toml::Value::Boolean(b) => b.into(),
        toml::Value::Datetime(d) => d.to_string().into(),
        toml::Value::Array(a) => a.into_iter().map(|e| convert_value_to_dynamic(e)).collect::<rhai::Array>().into(),
        toml::Value::Table(t) => convert_table_to_map(t).into(),
    }
}

fn convert_table_to_map(table: Table) -> rhai::Map {
    let mut map = rhai::Map::new();
    for (k, v) in table {
        map.insert(k.into(), convert_value_to_dynamic(v));
    }
    map
}

pub fn from_file(file: &str) -> RhaiResult<rhai::Map> {
    let content = std::fs::read_to_string(file).unwrap();
    Ok(from_string(&content).unwrap())
}

pub fn from_string(content: &str) -> RhaiResult<rhai::Map> {
    Ok(convert_table_to_map(content.parse::<Table>().unwrap()))
}

pub fn register(engine: &mut Engine) {
    let mut module = Module::new();
    module.set_native_fn("from_file", from_file);
    module.set_native_fn("from_string", from_string);
    engine.register_static_module("toml", module.into());
}