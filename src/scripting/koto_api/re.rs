use koto::prelude::*;
use koto::runtime::Result;
use std::collections::HashMap;
use std::rc::Rc;

pub fn make_module() -> KMap {
    let result = KMap::with_type("re");
    result.add_fn("regex", |ctx| match ctx.args() {
        [Value::Str(pattern)] => Ok(Regex::new(pattern)?.into()),
        unexpected => type_error_with_slice("a regex pattern as string", unexpected),
    });
    result
}

#[derive(Clone, Debug)]
pub struct Regex(Rc<regex::Regex>);

#[derive(Clone, Debug)]
pub struct Matches {
    text: Rc<str>,
    matches: Vec<(usize, usize)>,
    last_index: usize,
}

#[derive(Clone, Debug)]
pub struct Match {
    text: Rc<str>,
    start: usize,
    end: usize,
}

#[derive(Clone, Debug)]
pub struct Captures {
    text: Rc<str>,
    captures: Vec<Option<(usize, usize)>>,
    byname: HashMap<Rc<str>, (usize, usize)>,
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Self> {
        match regex::Regex::new(pattern) {
            Ok(r) => Ok(Self(Rc::new(r))),
            Err(e) => runtime_error!("Failed to parse regex pattern: {e}"),
        }
    }
}

impl KotoType for Regex {
    const TYPE: &'static str = "Regex";
}

impl KotoObject for Regex {
    fn object_type(&self) -> KString {
        REGEX_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        self.clone().into()
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        REGEX_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Regex");
        Ok(())
    }
}

impl From<Regex> for Value {
    fn from(regex: Regex) -> Self {
        KObject::from(regex).into()
    }
}

fn make_regex_entries() -> ValueMap {
    ObjectEntryBuilder::<Regex>::new()
        .method("is_match", |ctx| match ctx.args {
            [Value::Str(text)] => Ok(ctx.instance()?.0.is_match(text).into()),
            unexpected => type_error_with_slice("a string", unexpected),
        })
        .method("find_all", |ctx| match ctx.args {
            [Value::Str(text)] => {
                let r = ctx.instance()?;
                let matches = r.0.find_iter(text);
                Ok(Matches {
                    text: Rc::from(text.as_str()),
                    matches: matches.map(|m| (m.start(), m.end())).collect(),
                    last_index: 0,
                }
                .into())
            }
            unexpected => type_error_with_slice("a string", unexpected),
        })
        .method("find", |ctx| match ctx.args {
            [Value::Str(text)] => {
                let r = ctx.instance()?;
                let m = r.0.find(text);
                match m {
                    Some(m) => Ok(Match::new(Rc::from(text.as_str()), m.start(), m.end()).into()),
                    None => Ok(Value::Null),
                }
            }
            unexpected => type_error_with_slice("a string", unexpected),
        })
        .method("captures", |ctx| match ctx.args {
            [Value::Str(text)] => {
                let r = ctx.instance()?;
                let captures = r.0.captures(text);
                let capture_names = r.0.capture_names();
                match captures {
                    Some(captures) => {
                        let mut byname = HashMap::new();
                        for name in capture_names {
                            if let Some(name) = name {
                                let m = captures.name(name).unwrap();
                                byname.insert(Rc::from(name), (m.start(), m.end()));
                            }
                        }

                        Ok(Captures {
                            text: Rc::from(text.as_str()),
                            captures: captures
                                .iter()
                                .map(|m| match m {
                                    Some(m) => Some((m.start(), m.end())),
                                    None => None,
                                })
                                .collect(),
                            byname,
                        }
                        .into())
                    }
                    None => Ok(Value::Null),
                }
            }
            unexpected => type_error_with_slice("a string", unexpected),
        })
        .method("replace_all", |ctx| match ctx.args {
            [Value::Str(text), Value::Str(replacement)] => {
                let r = ctx.instance()?;
                let result = r.0.replace_all(text, replacement.as_str());
                Ok(result.to_string().into())
            }
            unexpected => type_error_with_slice("two strings", unexpected),
        })
        .build()
}

thread_local! {
    static REGEX_TYPE_STRING: KString = Regex::TYPE.into();
    static REGEX_ENTRIES: ValueMap = make_regex_entries();
}

impl Matches {}

impl KotoType for Matches {
    const TYPE: &'static str = "Matches";
}

impl KotoObject for Matches {
    fn object_type(&self) -> KString {
        MATCHES_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        self.clone().into()
    }

    fn is_iterable(&self) -> IsIterable {
        IsIterable::ForwardIterator
    }

    fn iterator_next(&mut self, _vm: &mut Vm) -> Option<KIteratorOutput> {
        if self.last_index >= self.matches.len() {
            self.last_index = 0;
            return None;
        } else {
            let result = match self.matches.get(self.last_index) {
                Some((start, end)) => Some(KIteratorOutput::Value(
                    Match::new(self.text.as_ref().into(), *start, *end).into(),
                )),
                None => None,
            };

            self.last_index += 1;
            return result;
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Matches");
        Ok(())
    }
}

impl From<Matches> for Value {
    fn from(matches: Matches) -> Self {
        KObject::from(matches).into()
    }
}

thread_local! {
    static MATCHES_TYPE_STRING: KString = Matches::TYPE.into();
}

impl Match {
    pub fn new(text: Rc<str>, start: usize, end: usize) -> Self {
        Self { text, start, end }
    }

    pub fn text(&self) -> &str {
        &self.text[self.start..self.end]
    }

    pub fn range(&self) -> Value {
        KRange::bounded(self.start.try_into().unwrap(), self.end.try_into().unwrap(), false).into()
    }
}

impl KotoType for Match {
    const TYPE: &'static str = "Match";
}

impl KotoObject for Match {
    fn object_type(&self) -> KString {
        MATCH_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        self.clone().into()
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        MATCH_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Match");
        Ok(())
    }
}

impl From<Match> for Value {
    fn from(match_: Match) -> Self {
        KObject::from(match_).into()
    }
}

fn make_match_entries() -> ValueMap {
    ObjectEntryBuilder::<Match>::new()
        .method("text", |ctx| {
            let m = ctx.instance()?;
            Ok(m.text().into())
        })
        .method("start", |ctx| {
            let m = ctx.instance()?;
            Ok(m.start.into())
        })
        .method("end", |ctx| {
            let m = ctx.instance()?;
            Ok(m.end.into())
        })
        .method("range", |ctx| {
            let m = ctx.instance()?;
            Ok(m.range().into())
        })
        .build()
}

thread_local! {
    static MATCH_TYPE_STRING: KString = Match::TYPE.into();
    static MATCH_ENTRIES: ValueMap = make_match_entries();
}

impl KotoType for Captures {
    const TYPE: &'static str = "Captures";
}

impl KotoObject for Captures {
    fn object_type(&self) -> KString {
        CAPTURES_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        self.clone().into()
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        CAPTURES_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn index(&self, index: &Value) -> Result<Value> {
        match index {
            Value::Number(index) => match self.captures.get(index.as_i64() as usize) {
                Some(Some((start, end))) => {
                    Ok(Match::new(self.text.clone(), *start, *end).into())
                }
                _ => runtime_error!("Invalid capture group index"),
            },
            Value::Str(name) => match self.byname.get(name.as_str()) {
                Some(m) => Ok(Match::new(self.text.clone(), m.0, m.1).into()),
                None => runtime_error!("Invalid capture group name"),
            },
            unexpected => type_error("Invalid index (must be Number or Str)", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Captures");
        Ok(())
    }
}

impl From<Captures> for Value {
    fn from(captures: Captures) -> Self {
        KObject::from(captures).into()
    }
}

fn make_captures_entries() -> ValueMap {
    ObjectEntryBuilder::<Captures>::new()
        .method("get", |ctx| match ctx.args {
            [Value::Number(index)] => {
                let c = ctx.instance()?;
                match c.captures.get(index.as_i64() as usize) {
                    Some(Some((start, end))) => Ok(Match::new(c.text.clone(), *start, *end).into()),
                    _ => Ok(Value::Null),
                }
            }
            [Value::Str(name)] => {
                let c = ctx.instance()?;
                match c.byname.get(name.as_str()) {
                    Some(m) => Ok(Match::new(c.text.clone(), m.0, m.1).into()),
                    None => Ok(Value::Null),
                }
            }
            unexpected => type_error_with_slice("a number", unexpected),
        })
        .method("len", |ctx| {
            let c = ctx.instance()?;
            Ok(c.captures.len().into())
        })
        .build()
}

thread_local! {
    static CAPTURES_TYPE_STRING: KString = Captures::TYPE.into();
    static CAPTURES_ENTRIES: ValueMap = make_captures_entries();
}
