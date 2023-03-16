// This is a mess. Need help ;)
use regex::Regex;
use rhai::{Engine, CustomType, TypeBuilder};
use super::RhaiResult;

#[derive(Clone)]
struct Re {
    regex: Regex,
}

impl Re {
    pub fn new(pattern: &str) -> Self {
        Self {
            regex: Regex::new(pattern).unwrap(),
        }
    }

    pub fn is_match(&mut self, text: &str) -> RhaiResult<bool> {
        Ok(self.regex.is_match(text))
    }

    pub fn replace_all(&mut self, text: &str, replacement: &str) -> RhaiResult<String> {
        Ok(self.regex.replace_all(text, replacement).into_owned())
    }

    pub fn find(&mut self, text: &str) -> rhai::Dynamic {
        if let Some(m) = self.regex.find(text) {
            rhai::Dynamic::from(Match::new(&m))
        } else {
            rhai::Dynamic::UNIT
        }
    }

    pub fn find_iter(&mut self, text: &str) -> RhaiResult<rhai::Array> {
        let mut array = rhai::Array::new();
        for m in self.regex.find_iter(text) {
            let mut map = rhai::Map::new();
            map.insert("start".into(), (m.start() as i64).into());
            map.insert("end".into(), (m.end() as i64).into());
            array.push(map.into());
        }
        Ok(array)
    }

    pub fn captures(&mut self, text: &str) -> rhai::Dynamic {
        if let Some(c) = self.regex.captures(text) {
            rhai::Dynamic::from(Captures::new(&c))
        } else {
            rhai::Dynamic::UNIT
        }
    }

    pub fn captures_iter(&mut self, text: &str) -> RhaiResult<rhai::Array> {
        let mut array = rhai::Array::new();
        for c in self.regex.captures_iter(text) {
            array.push(rhai::Dynamic::from(Captures::new(&c)));
        }
        Ok(array)
    }
}

#[derive(Clone)]
struct Match {
    start: i64,
    end: i64,
    text: String,
}

impl Match {
    pub fn new(m: &regex::Match) -> Self {
        Self {
            start: m.start() as i64,
            end: m.end() as i64,
            text: m.as_str().to_string(),
        }
    }

    pub fn start(&mut self) -> i64 {
        self.start
    }

    pub fn end(&mut self) -> i64 {
        self.end
    }

    pub fn as_str(&mut self) -> String {
        // My Rust skills are not good enough to get this to work without cloning again
        self.text.to_string()
    }
}

impl CustomType for Match {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Match")
            .with_fn("match", Self::new)
            .with_get("start", Self::start)
            .with_get("end", Self::end)
            .with_fn("as_str", Self::as_str);
    }
}

#[derive(Clone)]
struct Captures {
    c: Vec<Option<Match>>,
}

impl Captures {
    pub fn new(c: &regex::Captures) -> Self {
        Self {
            c: c.iter().map(|m| if let Some(m1) = m { Some(Match::new(&m1)) } else { None }).collect(),
        }
    }

    pub fn get(&mut self, index: i64) -> rhai::Dynamic {
        if let Some(m) = self.c.get(index as usize) {
            if let Some(m1) = m {
                // Fix the unnecessary clone
                rhai::Dynamic::from(m1.clone())
            } else {
                rhai::Dynamic::UNIT
            }
        } else {
            rhai::Dynamic::UNIT
        }
    }
}

impl CustomType for Captures {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Captures")
            .with_fn("captures", Self::new)
            .with_fn("get", Self::get);
    }
}

pub fn register(engine: &mut Engine) {
    engine.register_fn("regex", Re::new);
    engine.register_fn("is_match", Re::is_match);
    engine.register_fn("replace_all", Re::replace_all);
    engine.register_fn("find", Re::find);
    engine.register_fn("find_iter", Re::find_iter);
    engine.register_fn("captures", Re::captures);
    engine.register_fn("captures_iter", Re::captures_iter);
    engine.build_type::<Match>();
    engine.build_type::<Captures>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        let mut engine = Engine::new();
        register(&mut engine);

        assert!(engine.eval::<bool>(r#"let p = regex("\\d+"); p.is_match("123")"#).unwrap());
        assert_eq!(engine.eval::<String>(r#"let p = regex("\\d+"); p.replace_all("123", "abc")"#).unwrap(), "abc");
        assert_eq!(engine.eval::<i64>(r#"let p = regex("\\d+"); p.find("123").start"#).unwrap(), 0);
        assert_eq!(engine.eval::<i64>(r#"let p = regex("\\d"); let it = p.find_iter("123"); it.len"#).unwrap(), 3);
        assert_eq!(engine.eval::<i64>(r#"let p = regex("(\\d)(\\d)(\\d)"); let c = p.captures("123"); c.get(3).start"#).unwrap(), 2);
        assert_eq!(engine.eval::<i64>(r#"let p = regex("(\\d)"); let it = p.captures_iter("123"); it.len"#).unwrap(), 3);
        //assert_eq!(engine.eval::<i64>(r#"let p = regex("(\\d)(\\d)(\\d)"); let it = p.captures_iter("123"); it.get(1).get(2).start"#).unwrap(), 1);
    }
}