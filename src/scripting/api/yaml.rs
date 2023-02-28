use rhai::{Engine, Dynamic, Module};
use super::RhaiResult;
use yaml_rust::{YamlLoader, Yaml, yaml::Hash};

fn convert_yaml_to_dynamic(yaml: Yaml) -> Dynamic {
    match yaml {
        Yaml::String(s) => s.into(),
        Yaml::Integer(i) => i.into(),
        Yaml::Real(f) => f.parse::<f64>().unwrap().into(),
        Yaml::Boolean(b) => b.into(),
        Yaml::Array(a) => convert_vector_to_array(a).into(),
        Yaml::Hash(t) => convert_hash_to_map(t).into(),
        _ => "".into(),
    }
}

fn convert_hash_to_map(hash: Hash) -> rhai::Map {
    let mut map = rhai::Map::new();
    for (k, v) in hash {
        map.insert(k.as_str().unwrap().into(), convert_yaml_to_dynamic(v));
    }
    map
}

fn convert_vector_to_array(yaml: Vec<Yaml>) -> rhai::Array {
    yaml.into_iter().map(|e| convert_yaml_to_dynamic(e)).collect::<rhai::Array>()
}

pub fn from_file(file: &str) -> RhaiResult<rhai::Array> {
    let content = std::fs::read_to_string(file).unwrap();
    Ok(from_string(&content).unwrap())
}

pub fn from_string(content: &str) -> RhaiResult<rhai::Array> {
    Ok(convert_vector_to_array(YamlLoader::load_from_str(content).unwrap()))
}

pub fn register(engine: &mut Engine) {
    let mut module = Module::new();
    module.set_native_fn("from_file", from_file);
    module.set_native_fn("from_string", from_string);
    engine.register_static_module("yaml", module.into());
}