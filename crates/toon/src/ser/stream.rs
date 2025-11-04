use serde::ser::*;
use serde::Serialize;

#[cfg(feature = "json")]
use serde_json::Value;

use crate::encode::{primitives, writer::LineWriter};
use crate::options::Options;
#[cfg(not(feature = "json"))]
use crate::value::Value as IValue;

#[cfg(not(feature = "std"))]
use alloc::{format, string::{String, ToString}, vec::Vec};

#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

pub fn to_string_streaming<T: Serialize>(value: &T, options: &Options) -> crate::Result<String> {
    let mut w = LineWriter::new();
    let mut ser = StreamingSerializer { w: &mut w, opts: options, indent: 0 };
    value.serialize(&mut ser).map_err(|e| crate::error::Error::Message(e.to_string()))?;
    Ok(w.into_string())
}

#[derive(Debug)]
struct SerError { msg: String }

impl core::fmt::Display for SerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.msg)
    }
}
impl serde::ser::Error for SerError {
    fn custom<T: core::fmt::Display>(t: T) -> Self { SerError { msg: format!("{}", t) } }
}

impl core::error::Error for SerError {}

struct StreamingSerializer<'a> {
    w: &'a mut LineWriter,
    opts: &'a Options,
    indent: usize,
}

impl<'a> StreamingSerializer<'a> {
    fn with_indent<'b>(&'b mut self, indent: usize) -> StreamingSerializer<'b> {
        StreamingSerializer { w: self.w, opts: self.opts, indent }
    }
}

impl<'a, 'de> Serializer for &'a mut StreamingSerializer<'de> {
    type Ok = ();
    type Error = SerError;

    #[cfg(feature = "json")]
    type SerializeSeq = SeqSer<'a, 'de>;
    #[cfg(not(feature = "json"))]
    type SerializeSeq = SeqSerAlloc<'a, 'de>;

    #[cfg(feature = "json")]
    type SerializeTuple = SeqSer<'a, 'de>;
    #[cfg(not(feature = "json"))]
    type SerializeTuple = SeqSerAlloc<'a, 'de>;

    #[cfg(feature = "json")]
    type SerializeTupleStruct = SeqSer<'a, 'de>;
    #[cfg(not(feature = "json"))]
    type SerializeTupleStruct = SeqSerAlloc<'a, 'de>;

    #[cfg(feature = "json")]
    type SerializeTupleVariant = SeqSer<'a, 'de>;
    #[cfg(not(feature = "json"))]
    type SerializeTupleVariant = SeqSerAlloc<'a, 'de>;

    type SerializeMap = MapSer<'a, 'de>;
    type SerializeStruct = MapSer<'a, 'de>;
    type SerializeStructVariant = MapSer<'a, 'de>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, primitives::format_bool(v))) }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, &v.to_string())) }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, &v.to_string())) }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, &v.to_string())) }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, &v.to_string())) }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, &v.to_string())) }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, &v.to_string())) }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, &v.to_string())) }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, &v.to_string())) }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let f = v as f64;
        if f.is_finite() { Ok(self.w.line(self.indent, &f.to_string())) }
        else if f.is_nan() { Ok(self.w.line(self.indent, &primitives::escape_and_quote("NaN"))) }
        else if f.is_sign_positive() { Ok(self.w.line(self.indent, &primitives::escape_and_quote("Infinity"))) }
        else { Ok(self.w.line(self.indent, &primitives::escape_and_quote("-Infinity"))) }
    }
    fn serialize_f64(self, f: f64) -> Result<Self::Ok, Self::Error> {
        if f.is_finite() { Ok(self.w.line(self.indent, &f.to_string())) }
        else if f.is_nan() { Ok(self.w.line(self.indent, &primitives::escape_and_quote("NaN"))) }
        else if f.is_sign_positive() { Ok(self.w.line(self.indent, &primitives::escape_and_quote("Infinity"))) }
        else { Ok(self.w.line(self.indent, &primitives::escape_and_quote("-Infinity"))) }
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, &primitives::format_string(&v.to_string(), self.opts.delimiter))) }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, &primitives::format_string(v, self.opts.delimiter))) }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for b in v { SerializeSeq::serialize_element(&mut seq, b)?; }
        SerializeSeq::end(seq)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, primitives::format_null())) }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> { value.serialize(self) }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> { Ok(self.w.line(self.indent, primitives::format_null())) }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> { self.serialize_unit() }
    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> { self.serialize_str(variant) }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _name: &'static str, value: &T) -> Result<Self::Ok, Self::Error> { value.serialize(self) }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(self, _name: &'static str, _variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error> {
        self.w.line_key_only(self.indent, &primitives::format_string(variant, self.opts.delimiter));
        let mut child = self.with_indent(self.indent + 2);
        value.serialize(&mut child)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        #[cfg(feature = "json")]
        { return Ok(SeqSer { parent: self, items: Vec::new() }); }
        #[cfg(not(feature = "json"))]
        { return Ok(SeqSerAlloc { parent: self, items: Vec::new() }); }
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> { self.serialize_seq(Some(len)) }
    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> { self.serialize_seq(Some(len)) }
    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> { self.serialize_tuple_struct(variant, len) }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> { Ok(MapSer { parent: self, next_key: None }) }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> { self.serialize_map(None) }
    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.w.line_key_only(self.indent, &primitives::format_string(variant, self.opts.delimiter));
        Ok(MapSer { parent: self, next_key: None })
    }
}

#[cfg(feature = "json")]
struct SeqSer<'a, 'de> { parent: &'a mut StreamingSerializer<'de>, items: Vec<Value> }

#[cfg(feature = "json")]
impl<'a, 'de> SerializeSeq for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let val = crate::ser::value_builder::to_value(value, self.parent.opts);
        self.items.push(val);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(keys) = crate::encode::encoders::is_tabular_array(&self.items) {
            let dch = primitives::delimiter_char(self.parent.opts.delimiter);
            let key_cells: Vec<String> = keys.iter().map(|k| primitives::format_string(k, self.parent.opts.delimiter)).collect();
            let header = join_with_delim(&key_cells, dch);
            self.parent.w.line(self.parent.indent, &format!("@{} {}", dch, header));
            for item in &self.items {
                let obj = item.as_object().unwrap();
                let mut cells: Vec<String> = Vec::with_capacity(keys.len());
                for k in &keys {
                    let v = obj.get(k).unwrap();
                    let cell = match v {
                        Value::Null => primitives::format_null().to_string(),
                        Value::Bool(b) => primitives::format_bool(*b).to_string(),
                        Value::Number(n) => n.to_string(),
                        Value::String(s) => primitives::format_string(s, self.parent.opts.delimiter),
                        _ => "null".to_string(),
                    };
                    cells.push(cell);
                }
                let row = join_with_delim(&cells, dch);
                self.parent.w.line_list_item(self.parent.indent, &row);
            }
            return Ok(());
        }
        for item in &self.items {
            match item {
                Value::Null => self.parent.w.line_list_item(self.parent.indent, primitives::format_null()),
                Value::Bool(b) => self.parent.w.line_list_item(self.parent.indent, primitives::format_bool(*b)),
                Value::Number(n) => self.parent.w.line_list_item(self.parent.indent, &n.to_string()),
                Value::String(s) => self.parent.w.line_list_item(self.parent.indent, &primitives::format_string(s, self.parent.opts.delimiter)),
                Value::Array(_) | Value::Object(_) => {
                    self.parent.w.line(self.parent.indent, "-");
                    let mut child = self.parent.with_indent(self.parent.indent + 2);
                    // Recurse by encoding Value
                    crate::encode::encoders::encode_value(item, child.w, child.opts, child.indent)
                        .map_err(|e| SerError::custom(e.to_string()))?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "json")]
impl<'a, 'de> SerializeTuple for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> { SerializeSeq::serialize_element(self, value) }
    fn end(self) -> Result<Self::Ok, Self::Error> { SerializeSeq::end(self) }
}

#[cfg(feature = "json")]
impl<'a, 'de> SerializeTupleStruct for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> { SerializeSeq::serialize_element(self, value) }
    fn end(self) -> Result<Self::Ok, Self::Error> { SerializeSeq::end(self) }
}

#[cfg(feature = "json")]
impl<'a, 'de> SerializeTupleVariant for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> { SerializeSeq::serialize_element(self, value) }
    fn end(self) -> Result<Self::Ok, Self::Error> { SerializeSeq::end(self) }
}

#[cfg(not(feature = "json"))]
struct SeqSerAlloc<'a, 'de> { parent: &'a mut StreamingSerializer<'de>, items: Vec<IValue> }

#[cfg(not(feature = "json"))]
impl<'a, 'de> SerializeSeq for SeqSerAlloc<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let v = crate::ser::value_builder_alloc::to_value(value, self.parent.opts);
        self.items.push(v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(keys) = is_tabular_array_alloc(&self.items) {
            let dch = primitives::delimiter_char(self.parent.opts.delimiter);
            let key_cells: Vec<String> = keys.iter().map(|k| primitives::format_string(k, self.parent.opts.delimiter)).collect();
            let header = join_with_delim(&key_cells, dch);
            self.parent.w.line(self.parent.indent, &format!("@{} {}", dch, header));
            for item in &self.items {
                let obj = match item { IValue::Object(pairs) => pairs, _ => unreachable!() };
                let mut cells: Vec<String> = Vec::with_capacity(keys.len());
                for k in &keys {
                    let v = obj.iter().find(|(kk, _)| kk == k).map(|(_, v)| v).unwrap();
                    let cell = match v {
                        IValue::Null => primitives::format_null().to_string(),
                        IValue::Bool(b) => primitives::format_bool(*b).to_string(),
                        IValue::Number(n) => n.to_string(),
                        IValue::String(s) => primitives::format_string(s, self.parent.opts.delimiter),
                        _ => "null".to_string(),
                    };
                    cells.push(cell);
                }
                let row = join_with_delim(&cells, dch);
                self.parent.w.line_list_item(self.parent.indent, &row);
            }
            return Ok(());
        }
        // Fallback: emit list
        for item in &self.items {
            match item {
                IValue::Null => self.parent.w.line_list_item(self.parent.indent, primitives::format_null()),
                IValue::Bool(b) => self.parent.w.line_list_item(self.parent.indent, primitives::format_bool(*b)),
                IValue::Number(n) => self.parent.w.line_list_item(self.parent.indent, &n.to_string()),
                IValue::String(s) => self.parent.w.line_list_item(self.parent.indent, &primitives::format_string(s, self.parent.opts.delimiter)),
                IValue::Array(_) | IValue::Object(_) => {
                    self.parent.w.line(self.parent.indent, "-");
                    let mut child = self.parent.with_indent(self.parent.indent + 2);
                    encode_internal_value_alloc(item, child.w, child.opts, child.indent)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(not(feature = "json"))]
impl<'a, 'de> SerializeTuple for SeqSerAlloc<'a, 'de> {
    type Ok = ();
    type Error = SerError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> { SerializeSeq::serialize_element(self, value) }
    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

#[cfg(not(feature = "json"))]
impl<'a, 'de> SerializeTupleStruct for SeqSerAlloc<'a, 'de> {
    type Ok = ();
    type Error = SerError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> { SerializeSeq::serialize_element(self, value) }
    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

#[cfg(not(feature = "json"))]
impl<'a, 'de> SerializeTupleVariant for SeqSerAlloc<'a, 'de> {
    type Ok = ();
    type Error = SerError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> { SerializeSeq::serialize_element(self, value) }
    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

struct MapSer<'a, 'de> { parent: &'a mut StreamingSerializer<'de>, next_key: Option<String> }

impl<'a, 'de> SerializeMap for MapSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let s = key_to_string(key, self.parent.opts)?;
        self.next_key = Some(s);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self.next_key.take().unwrap_or_default();
        let key_fmt = primitives::format_string(&key, self.parent.opts.delimiter);
        if let Ok(sv) = try_scalar_to_string(value, self.parent.opts) {
            self.parent.w.line_kv(self.parent.indent, &key_fmt, &sv);
        } else {
            self.parent.w.line_key_only(self.parent.indent, &key_fmt);
            let mut child = self.with_indent(self.parent.indent + 2);
            value.serialize(&mut child)?;
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

impl<'a, 'de> MapSer<'a, 'de> {
    fn with_indent<'b>(&'b mut self, indent: usize) -> StreamingSerializer<'b> {
        self.parent.with_indent(indent)
    }
}

impl<'a, 'de> SerializeStruct for MapSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> {
        SerializeMap::serialize_key(self, &key)?;
        SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

impl<'a, 'de> SerializeStructVariant for MapSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { SerializeStruct::end(self) }
}

fn join_with_delim(cells: &[String], dch: char) -> String {
    if dch == '\t' { cells.join("\t") } else { cells.join(&format!("{} ", dch)) }
}

// --- helpers for non-json path ---
#[cfg(not(feature = "json"))]
fn is_tabular_array_alloc(arr: &[IValue]) -> Option<Vec<String>> {
    if arr.is_empty() { return None; }
    let mut keys: Option<Vec<String>> = None;
    for v in arr {
        let obj = match v { IValue::Object(pairs) => pairs, _ => return None };
        let mut kset: Vec<String> = obj.iter().map(|(k, _)| k.clone()).collect();
        kset.sort();
        if let Some(ref ks) = keys {
            if *ks != kset { return None; }
        } else {
            keys = Some(kset);
        }
        for (_, vv) in obj.iter() {
            if !vv.is_primitive() { return None; }
        }
    }
    keys
}

#[cfg(not(feature = "json"))]
fn encode_internal_value_alloc(v: &IValue, w: &mut LineWriter, opts: &Options, indent: usize) -> Result<(), SerError> {
    match v {
        IValue::Null => { w.line(indent, primitives::format_null()); }
        IValue::Bool(b) => { w.line(indent, primitives::format_bool(*b)); }
        IValue::Number(n) => { w.line(indent, &n.to_string()); }
        IValue::String(s) => { w.line(indent, &primitives::format_string(s, opts.delimiter)); }
        IValue::Array(items) => {
            for item in items {
                if item.is_primitive() {
                    match item {
                        IValue::Null => w.line_list_item(indent, primitives::format_null()),
                        IValue::Bool(b) => w.line_list_item(indent, primitives::format_bool(*b)),
                        IValue::Number(n) => w.line_list_item(indent, &n.to_string()),
                        IValue::String(s) => w.line_list_item(indent, &primitives::format_string(s, opts.delimiter)),
                        _ => {}
                    }
                } else {
                    w.line(indent, "-");
                    encode_internal_value_alloc(item, w, opts, indent + 2)?;
                }
            }
        }
        IValue::Object(pairs) => {
            for (k, val) in pairs {
                let key = primitives::format_string(k, opts.delimiter);
                if val.is_primitive() {
                    match val {
                        IValue::Null => w.line_kv(indent, &key, primitives::format_null()),
                        IValue::Bool(b) => w.line_kv(indent, &key, primitives::format_bool(*b)),
                        IValue::Number(n) => w.line_kv(indent, &key, &n.to_string()),
                        IValue::String(s) => w.line_kv(indent, &key, &primitives::format_string(s, opts.delimiter)),
                        _ => {}
                    }
                } else {
                    w.line_key_only(indent, &key);
                    encode_internal_value_alloc(val, w, opts, indent + 2)?;
                }
            }
        }
    }
    Ok(())
}

fn try_scalar_to_string<T: ?Sized + Serialize>(value: &T, opts: &Options) -> Result<String, SerError> {
    let mut s = ScalarSerializer { out: None, opts };
    match value.serialize(&mut s) {
        Ok(()) => s.out.ok_or_else(|| SerError::custom("no scalar")),
        Err(e) => Err(e),
    }
}

struct ScalarSerializer<'a> { out: Option<String>, opts: &'a Options }

impl<'a, 'de> Serializer for &'a mut ScalarSerializer<'de> {
    type Ok = ();
    type Error = SerError;
    type SerializeSeq = Impossible<(), SerError>;
    type SerializeTuple = Impossible<(), SerError>;
    type SerializeTupleStruct = Impossible<(), SerError>;
    type SerializeTupleVariant = Impossible<(), SerError>;
    type SerializeMap = Impossible<(), SerError>;
    type SerializeStruct = Impossible<(), SerError>;
    type SerializeStructVariant = Impossible<(), SerError>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> { self.out = Some(primitives::format_bool(v).to_string()); Ok(()) }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> { self.serialize_f64(v as f64) }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        if v.is_finite() { self.out = Some(v.to_string()); }
        else if v.is_nan() { self.out = Some(primitives::escape_and_quote("NaN")); }
        else if v.is_sign_positive() { self.out = Some(primitives::escape_and_quote("Infinity")); }
        else { self.out = Some(primitives::escape_and_quote("-Infinity")); }
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> { self.out = Some(primitives::format_string(&v.to_string(), self.opts.delimiter)); Ok(()) }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> { self.out = Some(primitives::format_string(v, self.opts.delimiter)); Ok(()) }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> { Err(SerError::custom("non-scalar")) }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> { self.out = Some(primitives::format_null().to_string()); Ok(()) }
    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> { Err(SerError::custom("non-scalar")) }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> { self.out = Some(primitives::format_null().to_string()); Ok(()) }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> { self.serialize_unit() }
    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> { self.serialize_str(variant) }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _name: &'static str, _value: &T) -> Result<Self::Ok, Self::Error> { Err(SerError::custom("non-scalar")) }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T) -> Result<Self::Ok, Self::Error> { Err(SerError::custom("non-scalar")) }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> { Err(SerError::custom("non-scalar")) }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> { Err(SerError::custom("non-scalar")) }
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> { Err(SerError::custom("non-scalar")) }
    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> { Err(SerError::custom("non-scalar")) }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> { Err(SerError::custom("non-scalar")) }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> { Err(SerError::custom("non-scalar")) }
    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant, Self::Error> { Err(SerError::custom("non-scalar")) }
}

fn key_to_string<T: ?Sized + Serialize>(key: &T, _opts: &Options) -> Result<String, SerError> {
    let mut s = KeySerializer { out: None };
    key.serialize(&mut s)?;
    s.out.ok_or_else(|| SerError::custom("invalid key"))
}

struct KeySerializer { out: Option<String> }

impl<'de> Serializer for &mut KeySerializer {
    type Ok = ();
    type Error = SerError;
    type SerializeSeq = Impossible<(), SerError>;
    type SerializeTuple = Impossible<(), SerError>;
    type SerializeTupleStruct = Impossible<(), SerError>;
    type SerializeTupleVariant = Impossible<(), SerError>;
    type SerializeMap = Impossible<(), SerError>;
    type SerializeStruct = Impossible<(), SerError>;
    type SerializeStructVariant = Impossible<(), SerError>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> { self.out = Some(if v { "true".into() } else { "false".into() }); Ok(()) }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> { self.out = Some((v as f64).to_string()); Ok(()) }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> { self.out = Some(v.to_string()); Ok(()) }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> { Err(SerError::custom("invalid key")) }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> { self.out = Some("null".into()); Ok(()) }
    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> { Err(SerError::custom("invalid key")) }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> { self.out = Some("null".into()); Ok(()) }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> { self.serialize_unit() }
    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> { self.serialize_str(variant) }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _name: &'static str, _value: &T) -> Result<Self::Ok, Self::Error> { Err(SerError::custom("invalid key")) }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T) -> Result<Self::Ok, Self::Error> { Err(SerError::custom("invalid key")) }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> { Err(SerError::custom("invalid key")) }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> { Err(SerError::custom("invalid key")) }
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> { Err(SerError::custom("invalid key")) }
    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> { Err(SerError::custom("invalid key")) }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> { Err(SerError::custom("invalid key")) }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> { Err(SerError::custom("invalid key")) }
    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant, Self::Error> { Err(SerError::custom("invalid key")) }
}
