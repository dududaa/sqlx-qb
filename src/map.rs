use std::collections::BTreeMap;
use std::fmt::Display;

#[derive(Clone)]
pub struct QueryMap<'q>(BTreeMap<&'q str, String>);

impl<'q> QueryMap<'q> {
    pub fn new() -> Self {
        let map = BTreeMap::new();
        QueryMap(map)
    }
    pub fn add(&mut self, key: &'q str, value: impl Display) {
        self.0.insert(key, value.to_string());
    }

    pub(crate) fn inner(&self) -> &BTreeMap<&'q str, String> {
        &self.0
    }

    pub(crate) fn arg(idx: usize) -> String {
        if !cfg!(feature = "mysql") {
            format!("${}", idx + 1)
        } else {
            "?".to_string()
        }
    }
}

impl<'q> Default for QueryMap<'q> {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! query_map {
    ( $( $key:literal : $value:expr ),* $(,)? ) => {{
        let mut map = QueryMap::new();
        $(
            map.add($key, $value);
        )*

        map
    }};
}
