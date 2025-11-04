use serde::ser::*;
use serde::Serialize;
use serde_json::Value;

use crate::encode::{encoders, primitives, writer::LineWriter};
use crate::options::Options;

pub fn to_string_streaming<T: Serialize>(value: &T, options: &Options) -> crate::Result<String> {
    let mut w = LineWriter::new();
    let mut ser = StreamingSerializer { w: &mut w, opts: options, indent: 0 };
    value.serialize(&mut ser).map_err(|e| crate::error::Error::Message(e.to_string()))?;
    Ok(w.into_string())
}

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
    type Error = serde_json::Error;
    type SerializeSeq = SeqSer<'a, 'de>;
    type SerializeTuple = SeqSer<'a, 'de>;
    type SerializeTupleStruct = SeqSer<'a, 'de>;
    type SerializeTupleVariant = SeqSer<'a, 'de>;
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
        let mut _child = self.with_indent(self.indent);
        let mut seq = self.serialize_seq(Some(v.len()))?;
for b in v { serde::ser::SerializeSeq::serialize_element(&mut seq, b)?; }
        serde::ser::SerializeSeq::end(seq)
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
        Ok(SeqSer { parent: self, items: Vec::new() })
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> { self.serialize_seq(Some(len)) }
    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> { self.serialize_seq(Some(len)) }
fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> { self.serialize_tuple_struct(variant, len) }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSer { parent: self, next_key: None })
    }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> { self.serialize_map(None) }
    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.w.line_key_only(self.indent, &primitives::format_string(variant, self.opts.delimiter));
        Ok(MapSer { parent: self, next_key: None })
    }
}

struct SeqSer<'a, 'de> { parent: &'a mut StreamingSerializer<'de>, items: Vec<Value> }

impl<'a, 'de> SerializeSeq for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = serde_json::Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        // Buffer as Value for potential tabular detection
        let val = crate::ser::value_builder::to_value(value, self.parent.opts);
        self.items.push(val);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(keys) = encoders::is_tabular_array(&self.items) {
            // Emit header and rows
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
        // Fallback: emit list
        for item in &self.items {
            match item {
                Value::Null => self.parent.w.line_list_item(self.parent.indent, primitives::format_null()),
                Value::Bool(b) => self.parent.w.line_list_item(self.parent.indent, primitives::format_bool(*b)),
                Value::Number(n) => self.parent.w.line_list_item(self.parent.indent, &n.to_string()),
                Value::String(s) => self.parent.w.line_list_item(self.parent.indent, &primitives::format_string(s, self.parent.opts.delimiter)),
                Value::Array(_) | Value::Object(_) => {
                    self.parent.w.line(self.parent.indent, "-");
let child = self.parent.with_indent(self.parent.indent + 2);
                    // Recurse by encoding Value
                    crate::encode::encoders::encode_value(item, child.w, child.opts, child.indent).map_err(|e| serde_json::Error::custom(e.to_string()))?;
                }
            }
        }
        Ok(())
    }
}

impl<'a, 'de> SerializeTuple for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = serde_json::Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> { SerializeSeq::serialize_element(self, value) }
    fn end(self) -> Result<Self::Ok, Self::Error> { SerializeSeq::end(self) }
}

impl<'a, 'de> SerializeTupleStruct for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = serde_json::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> { SerializeSeq::serialize_element(self, value) }
    fn end(self) -> Result<Self::Ok, Self::Error> { SerializeSeq::end(self) }
}

impl<'a, 'de> SerializeTupleVariant for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = serde_json::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> { SerializeSeq::serialize_element(self, value) }
    fn end(self) -> Result<Self::Ok, Self::Error> { SerializeSeq::end(self) }
}

struct MapSer<'a, 'de> { parent: &'a mut StreamingSerializer<'de>, next_key: Option<String> }

impl<'a, 'de> SerializeMap for MapSer<'a, 'de> {
    type Ok = ();
    type Error = serde_json::Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        // Turn key into string
        let v = crate::ser::value_builder::to_value(key, self.parent.opts);
        let s = match v {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => if b { "true".into() } else { "false".into() },
            Value::Null => "null".into(),
            other => other.to_string(),
        };
        self.next_key = Some(s);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self.next_key.take().unwrap_or_default();
        // Buffer value to decide primitive vs nested
        let v = crate::ser::value_builder::to_value(value, self.parent.opts);
        let key_fmt = primitives::format_string(&key, self.parent.opts.delimiter);
        match v {
            Value::Null => self.parent.w.line_kv(self.parent.indent, &key_fmt, primitives::format_null()),
            Value::Bool(b) => self.parent.w.line_kv(self.parent.indent, &key_fmt, primitives::format_bool(b)),
            Value::Number(n) => self.parent.w.line_kv(self.parent.indent, &key_fmt, &n.to_string()),
            Value::String(s) => self.parent.w.line_kv(self.parent.indent, &key_fmt, &primitives::format_string(&s, self.parent.opts.delimiter)),
            Value::Array(_) | Value::Object(_) => {
                self.parent.w.line_key_only(self.parent.indent, &key_fmt);
let child = self.parent.with_indent(self.parent.indent + 2);
                // Recurse by encoding Value to preserve complex structure
                crate::encode::encoders::encode_value(&v, child.w, child.opts, child.indent).map_err(|e| serde_json::Error::custom(e.to_string()))?;
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

impl<'a, 'de> SerializeStruct for MapSer<'a, 'de> {
    type Ok = ();
    type Error = serde_json::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> {
        SerializeMap::serialize_key(self, &key)?;
        SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

impl<'a, 'de> SerializeStructVariant for MapSer<'a, 'de> {
    type Ok = ();
    type Error = serde_json::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> std::result::Result<(), Self::Error> {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> { SerializeStruct::end(self) }
}

fn join_with_delim(cells: &[String], dch: char) -> String {
    if dch == '\t' { cells.join("\t") } else { cells.join(&format!("{} ", dch)) }
}
