use rune::runtime::{Object, Value};
use rune::{ContextError, Module};
use yaml_rust::{yaml::Hash, Yaml, YamlLoader};

fn convert_yaml_to_rune(yaml: Yaml) -> Value {
    match yaml {
        Yaml::String(s) => s.into(),
        Yaml::Integer(i) => i.into(),
        Yaml::Real(f) => f.parse::<f64>().unwrap().into(),
        Yaml::Boolean(b) => b.into(),
        Yaml::Array(a) => convert_vector_to_rune(a).into(),
        Yaml::Hash(t) => convert_hash_to_object(t).into(),
        _ => ().into(),
    }
}

fn convert_hash_to_object(hash: Hash) -> Object {
    let mut map = Object::new();
    for (k, v) in hash {
        map.insert(k.as_str().unwrap().into(), convert_yaml_to_rune(v));
    }
    map
}

fn convert_vector_to_rune(yaml: Vec<Yaml>) -> rune::runtime::Vec {
    let mut vec = rune::runtime::Vec::new();
    for e in yaml {
        vec.push(convert_yaml_to_rune(e));
    }
    vec
}

pub fn from_file(file: &str) -> rune::runtime::Vec {
    let content = std::fs::read_to_string(file).unwrap();
    from_string(&content)
}

pub fn from_string(content: &str) -> rune::runtime::Vec {
    convert_vector_to_rune(YamlLoader::load_from_str(content).unwrap())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("yaml");
    module.function(["from_file"], from_file)?;
    module.function(["from_string"], from_string)?;
    Ok(module)
}
