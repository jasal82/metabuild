// This is a mess. Need help ;)
use regex::{CaptureMatches, Captures, Match, Matches, Regex};
use rune::{Any, ContextError, Module};

#[derive(Any)]
struct RegexRune {
    inner: Regex,
}

#[derive(Any, Clone)]
struct MatchRune {
    start: usize,
    end: usize,
    text: String,
}

#[derive(Any)]
struct CapturesRune {
    groups: Vec<Option<MatchRune>>
}

impl RegexRune {
    pub fn new(pattern: &str) -> Self {
        Self {
            inner: Regex::new(pattern).unwrap(),
        }
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.inner.is_match(text)
    }

    pub fn replace_all(&self, text: &str, replacement: &str) -> String {
        self.inner.replace_all(text, replacement).to_string()
    }

    pub fn find(&self, text: &str) -> Option<MatchRune> {
        if let Some(m) = self.inner.find(text) {
            Some(MatchRune {
                start: m.start(),
                end: m.end(),
                text: m.as_str().to_string(),
            })
        } else {
            None
        }
    }

    pub fn find_iter(&self, text: &str) -> Vec<MatchRune> {
        self.inner.find_iter(text).map(|m| MatchRune {
            start: m.start(),
            end: m.end(),
            text: m.as_str().to_string(),
        }).collect()
    }

    pub fn captures(&self, text: &str) -> Option<CapturesRune> {
        if let Some(c) = self.inner.captures(text) {
            Some(CapturesRune {
                groups: c.iter().map(|m| m.map(|m| MatchRune {
                    start: m.start(),
                    end: m.end(),
                    text: m.as_str().to_string(),
                })).collect(),
            })
        } else {
            None
        }
    }

    pub fn captures_iter(&self, text: &str) -> Vec<Option<CapturesRune>> {
        let mut result = Vec::new();
        for caps in self.inner.captures_iter(text) {
            
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

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn as_str(&self) -> String {
        self.text.clone()
    }
}

impl CapturesRune {
    pub fn new(c: &regex::Captures) -> Self {
        Self {
            groups: c.iter().map(|m| m.map(|m| MatchRune {
                start: m.start(),
                end: m.end(),
                text: m.as_str().to_string(),
            })).collect(),
        }
    }

    pub fn get(&self, index: usize) -> Option<MatchRune> {
        self.groups.get(index).unwrap().as_ref().map(|m| m.clone())
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("regex");
    module.ty::<RegexRune>()?;
    module.ty::<MatchRune>()?;
    module.ty::<CapturesRune>()?;

    module.function(["regex", "new"], RegexRune::new)?;
    module.inst_fn("is_match", RegexRune::is_match)?;
    module.inst_fn("replace_all", RegexRune::replace_all)?;
    module.inst_fn("find", RegexRune::find)?;
    module.inst_fn("find_iter", RegexRune::find_iter)?;
    module.inst_fn("captures", RegexRune::captures)?;
    module.inst_fn("captures_iter", RegexRune::captures_iter)?;

    module.inst_fn("start", MatchRune::start)?;
    module.inst_fn("end", MatchRune::end)?;
    module.inst_fn("as_str", MatchRune::as_str)?;
    module.inst_fn("get", CapturesRune::get)?;
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