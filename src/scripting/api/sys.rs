use koto::prelude::*;

pub fn make_module() -> KMap {
    let result = KMap::with_type("sys");

    result.add_fn("is_windows", |_| Ok(cfg!(target_os = "windows").into()));
    result.add_fn("is_mingw", |_| {
        Ok((cfg!(target_os = "windows") && std::env::vars().any(|(k, _)| k == "MSYSTEM")).into())
    });
    result.add_fn("is_linux", |_| Ok(cfg!(target_os = "linux").into()));
    result.add_fn("args", |_| {
        let list = std::env::args()
            .map(|a| KValue::Str(a.as_str().into()))
            .collect();
        Ok(KValue::List(KList::with_data(list)))
    });
    result.add_fn("env", |_| {
        let map = KMap::with_capacity(std::env::vars().count());
        for (k, v) in std::env::vars() {
            map.insert(KString::from(k), KValue::Str(v.as_str().into()));
        }
        Ok(KValue::Map(map))
    });

    result
}
