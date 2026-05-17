use crate::value::QbValue;
use std::collections::BTreeMap;

pub struct QueryMap<'q>(BTreeMap<&'q str, QbValue<'q>>);

impl<'q> QueryMap<'q> {
    pub fn new() -> Self {
        let map = BTreeMap::new();
        QueryMap(map)
    }
    pub fn add(&mut self, key: &'q str, value: impl Into<QbValue<'q>>) {
        self.0.insert(key, value.into());
    }

    pub(crate) fn inner(&self) -> &BTreeMap<&'q str, QbValue<'q>> {
        &self.0
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