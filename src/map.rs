use crate::model::QueryMapInput;
use crate::prelude::ModelInsert;
use std::collections::BTreeMap;
use std::fmt::Display;
#[cfg(feature = "serde")]
use {serde::Serialize, serde_json::Value, sqlx::Error};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct QueryMap(BTreeMap<String, String>);

impl QueryMap {
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

#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct MapInput {
    map: QueryMap,
    table_name: Option<String>,
}

impl MapInput {
    pub fn new(table_name: Option<String>) -> Self {
        Self {
            table_name,
            map: QueryMap::new(),
        }
    }

    pub fn add(&mut self, key: impl Display, value: impl Display) {
        self.map.add(key.to_string(), value.to_string());
    }

    pub fn with_map(mut self, map: QueryMap) -> Self {
        self.map = map;
        self
    }

    #[cfg(feature = "serde")]
    pub fn from_value<T: Serialize>(value: &T) -> Result<Self, Error> {
        let json_value = serde_json::to_value(value).map_err(|_| Error::BeginFailed)?;
        match json_value {
            Value::Object(map) => {
                let table_name = map
                    .get("table_name")
                    .map(|s| s.as_str())
                    .unwrap_or_default();

                let map_value = map
                    .get("map")
                    .ok_or(Error::ColumnNotFound("map input missing map".to_string()))?;

                let map = QueryMap::from_value(map_value)?;

                Ok(Self {
                    table_name: table_name.map(|s| s.to_string()),
                    map,
                })
            }
            _ => Err(Error::BeginFailed),
        }
    }
}

#[cfg(not(feature = "serde"))]
impl<'q, R> QueryMapInput<'q, R> for MapInput {
    fn table_name(&self) -> Option<String> {
        self.table_name.clone()
    }

    fn to_map(&'q self) -> Result<QueryMap, sqlx::Error> {
        Ok(self.map.clone())
    }
}

#[macro_export]
macro_rules! query_map {
    ( $( $key:literal : $value:expr ),* $(,)? ) => {{
        let mut input = MapInput::new(None);

        $(
            input.add($key, $value);
        )*

        input
    }};

    ( $( $table_name:expr, $key:literal : $value:expr ),* $(,)? ) => {{
        let mut input = MapInput::new($table_name);

        $(
            input.add($key, $value);
        )*

        input
    }};
}

#[macro_export]
macro_rules! json_map {
    ( $table_name:expr, $( $key:literal : $value:expr ),* $(,)? ) => {{
        let data = json! ({
          $( $key : $value ),*
        });

        QueryMap::from_value(&data).map(|map| MapInput::new(Some($table_name)).with_map(map))
    }};

    ( $( $key:literal : $value:expr ),* $(,)? ) => {{
        let data = json! ({
          $( $key : $value ),*
        });

        QueryMap::from_value(&data).map(|map| MapInput::new(None).with_map(map))
    }};
}

#[macro_export]
macro_rules! impl_type_for_query_map {
    () => {
        impl<'q, Returns> ModelInsert<'q, Returns> for MapInput {}

        #[cfg(feature = "serde")]
        impl<'q, Returns, T: Serialize + ModelInsert<'q, Returns>> QueryMapInput<'q, Returns>
            for T
        {
            fn table_name(&'q self) -> Option<String> {
                Self::TABLE_NAME.map(|s| s.to_string())
            }

            fn to_map(&'q self) -> Result<QueryMap, Error> {
                // MapInput::from_value works for json input
                match MapInput::from_value(self) {
                    Ok(input) => Ok(input.map),
                    // QueryMap::from_value works for struct input
                    _ => QueryMap::from_value(self)
                }
            }
        }
    };
}

// #[cfg(not(feature = "serde"))]
impl_type_for_query_map!();

// #[cfg(feature = "serde")]
// impl<'q,> ModelInsert<'q, ()> for MapInput {}
