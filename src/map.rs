use std::collections::BTreeMap;
use std::fmt::Display;

#[cfg(feature = "serde")]
use {serde::Serialize, serde_json::Value, sqlx::Error};

#[cfg(not(feature = "serde"))]
#[derive(Clone)]
pub struct QueryMap(BTreeMap<String, String>);

#[derive(Clone)]
#[cfg(feature = "serde")]
#[derive(Serialize)]
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

impl Default for QueryMap {
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
