// This is a mess. Need help ;)
use regex::Regex;
use rune::{Any, ContextError, Module};

#[derive(Any)]
#[rune(item = ::regex, name = Regex)]
struct RegexRune {
    inner: Regex,
}

#[derive(Any, Clone)]
#[rune(item = ::regex)]
struct MatchRune {
    start: usize,
    end: usize,
    text: String,
}

#[derive(Any)]
#[rune(item = ::regex)]
struct CapturesRune {
    groups: Vec<Option<MatchRune>>,
}

impl RegexRune {
    #[rune::function(path = Self::new)]
    pub fn new(pattern: &str) -> Self {
        Self {
            inner: Regex::new(pattern).unwrap(),
        }
    }

    #[rune::function]
    pub fn is_match(&self, text: &str) -> bool {
        self.inner.is_match(text)
    }

    #[rune::function]
    pub fn replace_all(&self, text: &str, replacement: &str) -> String {
        self.inner.replace_all(text, replacement).to_string()
    }

    #[rune::function]
    pub fn find(&self, text: &str) -> Option<MatchRune> {
        self.inner.find(text).map(|m| MatchRune {
            start: m.start(),
            end: m.end(),
            text: m.as_str().to_string(),
        })
    }

    #[rune::function]
    pub fn find_iter(&self, text: &str) -> Vec<MatchRune> {
        self.inner
            .find_iter(text)
            .map(|m| MatchRune {
                start: m.start(),
                end: m.end(),
                text: m.as_str().to_string(),
            })
            .collect()
    }

    #[rune::function]
    pub fn captures(&self, text: &str) -> Option<CapturesRune> {
        self.inner.captures(text).map(|c| CapturesRune::new(&c))
    }

    #[rune::function]
    pub fn captures_iter(&self, text: &str) -> Vec<Option<CapturesRune>> {
        let mut result = Vec::new();
        for caps in self.inner.captures_iter(text) {
            result.push(Some(CapturesRune::new(&caps)));
        }
        result
    }
}

impl MatchRune {
    pub fn new(m: &regex::Match) -> Self {
        Self {
            start: m.start(),
            end: m.end(),
            text: m.as_str().to_string(),
        }
    }

    #[rune::function]
    pub fn start(&self) -> usize {
        self.start
    }

    #[rune::function]
    pub fn end(&self) -> usize {
        self.end
    }

    #[rune::function]
    pub fn as_str(&self) -> String {
        self.text.clone()
    }
}

impl CapturesRune {
    pub fn new(c: &regex::Captures) -> Self {
        Self {
            groups: c.iter().map(|m| m.map(|m| MatchRune::new(&m))).collect(),
        }
    }

    #[rune::function]
    pub fn get(&self, index: usize) -> Option<MatchRune> {
        self.groups.get(index).unwrap().as_ref().cloned()
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("regex")?;
    module.ty::<RegexRune>()?;
    module.ty::<MatchRune>()?;
    module.ty::<CapturesRune>()?;

    module.function_meta(RegexRune::new)?;
    module.function_meta(RegexRune::is_match)?;
    module.function_meta(RegexRune::replace_all)?;
    module.function_meta(RegexRune::find)?;
    module.function_meta(RegexRune::find_iter)?;
    module.function_meta(RegexRune::captures)?;
    module.function_meta(RegexRune::captures_iter)?;

    module.function_meta(MatchRune::start)?;
    module.function_meta(MatchRune::end)?;
    module.function_meta(MatchRune::as_str)?;
    
    module.function_meta(CapturesRune::get)?;

    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        /*let mut engine = Engine::new();
        register(&mut engine);

        assert!(engine.eval::<bool>(r#"let p = regex("\\d+"); p.is_match("123")"#).unwrap());
        assert_eq!(engine.eval::<String>(r#"let p = regex("\\d+"); p.replace_all("123", "abc")"#).unwrap(), "abc");
        assert_eq!(engine.eval::<i64>(r#"let p = regex("\\d+"); p.find("123").start"#).unwrap(), 0);
        assert_eq!(engine.eval::<i64>(r#"let p = regex("\\d"); let it = p.find_iter("123"); it.len"#).unwrap(), 3);
        assert_eq!(engine.eval::<i64>(r#"let p = regex("(\\d)(\\d)(\\d)"); let c = p.captures("123"); c.get(3).start"#).unwrap(), 2);
        assert_eq!(engine.eval::<i64>(r#"let p = regex("(\\d)"); let it = p.captures_iter("123"); it.len"#).unwrap(), 3);
        //assert_eq!(engine.eval::<i64>(r#"let p = regex("(\\d)(\\d)(\\d)"); let it = p.captures_iter("123"); it.get(1).get(2).start"#).unwrap(), 1);*/
    }
}
