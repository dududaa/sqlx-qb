use std::collections::BTreeMap;
use std::fmt::Display;

#[cfg(feature = "serde")]
use {serde::Serialize, serde_json::Value, sqlx::Error};
use crate::model::QueryMapInput;
use crate::prelude::ModelInsert;

#[cfg(not(feature = "serde"))]
#[derive(Clone)]
pub struct QueryMap(BTreeMap<String, String>);

#[derive(Clone)]
#[cfg(feature = "serde")]
#[derive(Serialize)]
pub struct QueryMap(BTreeMap<String, String>);

impl<'q> QueryMap {
    pub fn new() -> Self {
        let map = BTreeMap::new();
        QueryMap(map)
    }
    pub fn add(&mut self, key: impl Display, value: impl Display) {
        self.0.insert(key.to_string(), value.to_string());
    }

    #[cfg(feature = "serde")]
    pub fn from_value<T: Serialize>(value: &T) -> Result<Self, Error> {
        let json_value = serde_json::to_value(value).map_err(|_| Error::BeginFailed)?;
        match json_value {
            Value::Object(map) => {
                let map: BTreeMap<String, Value> = map.into_iter().collect();
                let inner = map
                    .iter()
                    .map(|(key, value)| {
                        let new_value: String = match value {
                            Value::String(s) => s.clone(),
                            _ => value.to_string(),
                        };

                        (key.to_string(), new_value)
                    })
                    .collect();

                let mut map = QueryMap::new();
                map.0 = inner;

                Ok(map)
            }
            _ => Err(Error::BeginFailed),
        }
    }

    pub(crate) fn inner(&self) -> &BTreeMap<String, String> {
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

pub struct MapInput<'q> {
    map: QueryMap,
    table_name: Option<&'q str>
}

impl<'q> MapInput<'q> {
    pub fn new(table_name: Option<&'q str>) -> Self {
        Self { table_name, map: QueryMap::new() }
    }

    pub fn add(&mut self, key: impl Display, value: impl Display) {
        self.map.add(key.to_string(), value.to_string());
    }
}

impl<'q> QueryMapInput<'q> for MapInput<'q> {
    fn table_name(&'q self) -> Option<&'q str> {
        self.table_name
    }

    fn to_map(&'q self) -> QueryMap {
        self.map.clone()
    }
}

// impl<'q> ModelInsert<'q, ()> for MapInput<'q> {}

#[macro_export]
macro_rules! query_map {
    ( $( $key:literal : $value:expr ),* $(,)? ) => {{
        let mut input = MapInput::new(None);

        $(
            input.add($key, $value);
        )*

        input
    }};
}

#[macro_export]
macro_rules! impl_type_for_query_map {
    () => {
        impl<'q, Returns> ModelInsert<'q, Returns> for MapInput<'q> {}
    };
}

impl_type_for_query_map!();
