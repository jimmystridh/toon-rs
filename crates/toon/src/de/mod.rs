//! serde::Deserializer implementation backed by internal Value (alloc-friendly)

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

#[cfg(all(feature = "de_direct", feature = "json"))]
use core::any::TypeId;

use serde::de::{self, DeserializeOwned, IntoDeserializer, MapAccess, SeqAccess};

use crate::value::{Number, Value};
use crate::{Result, options::Options};

#[cfg(feature = "de_direct")]
pub mod direct;

#[cfg(all(feature = "de_direct", feature = "json"))]
use serde_json::Value as JsonValue;

#[derive(Debug)]
pub struct DeError {
    msg: String,
}

impl core::fmt::Display for DeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.msg)
    }
}
impl de::Error for DeError {
    fn custom<T: core::fmt::Display>(t: T) -> Self {
        DeError {
            msg: format!("{}", t),
        }
    }
}
impl core::error::Error for DeError {}

pub struct Deserializer {
    value: Value,
}

impl Deserializer {
    pub fn from_value(value: Value) -> Self {
        Self { value }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Number(n) => match n {
                Number::I64(i) => visitor.visit_i64(i),
                Number::U64(u) => visitor.visit_u64(u),
                Number::F64(f) => visitor.visit_f64(f),
            },
            Value::String(s) => visitor.visit_string(s),
            Value::Array(arr) => {
                struct SA {
                    elems: Vec<Value>,
                    idx: usize,
                }
                impl<'de> SeqAccess<'de> for SA {
                    type Error = DeError;
                    fn next_element_seed<T>(
                        &mut self,
                        seed: T,
                    ) -> core::result::Result<Option<T::Value>, Self::Error>
                    where
                        T: de::DeserializeSeed<'de>,
                    {
                        if self.idx >= self.elems.len() {
                            return Ok(None);
                        }
                        let v = core::mem::replace(&mut self.elems[self.idx], Value::Null);
                        self.idx += 1;
                        let de = Deserializer { value: v };
                        seed.deserialize(de).map(Some)
                    }
                }
                visitor.visit_seq(SA { elems: arr, idx: 0 })
            }
            Value::Object(obj) => {
                struct MA {
                    entries: Vec<(String, Value)>,
                    idx: usize,
                    next_val: Option<Value>,
                }
                impl<'de> MapAccess<'de> for MA {
                    type Error = DeError;
                    fn next_key_seed<K>(
                        &mut self,
                        seed: K,
                    ) -> core::result::Result<Option<K::Value>, Self::Error>
                    where
                        K: de::DeserializeSeed<'de>,
                    {
                        if self.idx >= self.entries.len() {
                            return Ok(None);
                        }
                        let (ref key, ref val) = self.entries[self.idx];
                        let de_key = key.clone().into_deserializer();
                        self.next_val = Some(val.clone());
                        seed.deserialize(de_key).map(Some)
                    }
                    fn next_value_seed<VV>(
                        &mut self,
                        seed: VV,
                    ) -> core::result::Result<VV::Value, Self::Error>
                    where
                        VV: de::DeserializeSeed<'de>,
                    {
                        let v = self.next_val.take().unwrap_or(Value::Null);
                        self.idx += 1;
                        let de = Deserializer { value: v };
                        seed.deserialize(de)
                    }
                }
                visitor.visit_map(MA {
                    entries: obj,
                    idx: 0,
                    next_val: None,
                })
            }
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct
        enum identifier ignored_any
    }
}

pub fn from_str<T: DeserializeOwned + 'static>(s: &str, options: &Options) -> Result<T> {
    #[cfg(feature = "de_direct")]
    {
        #[cfg(feature = "json")]
        {
            if TypeId::of::<T>() == TypeId::of::<JsonValue>() {
                return from_str_via_internal_value(s, options);
            }
        }
        crate::de::direct::from_str(s, options)
    }

    #[cfg(not(feature = "de_direct"))]
    {
        from_str_via_internal_value(s, options)
    }
}

#[cfg_attr(all(feature = "de_direct", not(feature = "json")), allow(dead_code))]
fn from_str_via_internal_value<T: DeserializeOwned>(s: &str, options: &Options) -> Result<T> {
    let lines = crate::decode::scanner::scan(s);
    if options.strict {
        if let Err(e) = crate::decode::validation::validate_indentation(&lines) {
            return Err(crate::error::Error::Syntax {
                line: e.line,
                message: e.message,
            });
        }
    }
    let v = crate::decode::parser::parse_to_internal_value_from_lines(lines, options.strict)?;
    let deser = Deserializer::from_value(v);
    let t = T::deserialize(deser).map_err(|e: DeError| crate::error::Error::Message(e.msg))?;
    Ok(t)
}
