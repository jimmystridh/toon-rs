//! serde::Deserializer implementation backed by parsed Value

use serde::de::{self, DeserializeOwned, IntoDeserializer};
use serde_json::Value;

use crate::{options::Options, Result};

pub struct Deserializer {
    value: Value,
}

impl Deserializer {
    pub fn from_value(value: Value) -> Self { Self { value } }
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.value.into_deserializer().deserialize_any(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct
        enum identifier ignored_any
    }
}

pub fn from_str<T: DeserializeOwned>(s: &str, options: &Options) -> Result<T> {
    let v = crate::decode::parser::parse_to_value_with_strict(s, options.strict)?;
    let deser = Deserializer::from_value(v);
    let t = T::deserialize(deser).map_err(crate::error::Error::from)?;
    Ok(t)
}
