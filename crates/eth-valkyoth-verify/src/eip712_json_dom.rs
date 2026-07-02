use core::fmt;

use serde::de::{Error as _, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::string::{String, ToString};
use std::vec::Vec;

use super::Eip712JsonError;

pub(super) enum Json {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Json>),
    Object(Object),
}

pub(super) struct Object(Vec<(String, Json)>);

impl Object {
    pub(super) fn get(&self, name: &str) -> Result<&Json, Eip712JsonError> {
        self.get_optional(name).ok_or(Eip712JsonError::Shape)
    }

    pub(super) fn get_optional(&self, name: &str) -> Option<&Json> {
        self.0
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, value)| value)
    }

    pub(super) fn entries(&self) -> impl Iterator<Item = (&str, &Json)> {
        self.0.iter().map(|(name, value)| (name.as_str(), value))
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
}

impl Json {
    pub(super) fn as_object(&self) -> Result<&Object, Eip712JsonError> {
        match self {
            Self::Object(object) => Ok(object),
            _ => Err(Eip712JsonError::Shape),
        }
    }

    pub(super) fn as_array(&self) -> Result<&[Json], Eip712JsonError> {
        match self {
            Self::Array(values) => Ok(values),
            _ => Err(Eip712JsonError::Shape),
        }
    }

    pub(super) fn as_str(&self) -> Result<&str, Eip712JsonError> {
        match self {
            Self::String(value) => Ok(value),
            _ => Err(Eip712JsonError::Shape),
        }
    }

    pub(super) fn as_bool(&self) -> Result<bool, Eip712JsonError> {
        match self {
            Self::Bool(value) => Ok(*value),
            _ => Err(Eip712JsonError::Shape),
        }
    }
}

impl<'de> Deserialize<'de> for Json {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonVisitor)
    }
}

struct JsonVisitor;

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = Json;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Json::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Json::Number(serde_json::Number::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Json::Number(serde_json::Number::from(value)))
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(E::custom("floating point values are not admitted"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Json::String(value.to_string()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Json::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Json::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Json::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(Json::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entries = Vec::<(String, Json)>::new();
        while let Some((key, value)) = map.next_entry::<String, Json>()? {
            if entries.iter().any(|(existing, _)| existing == &key) {
                return Err(A::Error::custom("duplicate object key"));
            }
            entries.push((key, value));
        }
        Ok(Json::Object(Object(entries)))
    }
}
