use serde::Serialize;
use serde::ser::*;

#[cfg(feature = "json")]
use serde_json::Value;

use crate::encode::{primitives, writer::LineWriter};
use crate::options::Options;
#[cfg(not(feature = "json"))]
use crate::value::Value as IValue;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

pub fn to_string_streaming<T: Serialize>(value: &T, options: &Options) -> crate::Result<String> {
    let mut w = LineWriter::new();
    let mut ser = StreamingSerializer {
        w: &mut w,
        opts: options,
        indent: 0,
    };
    value
        .serialize(&mut ser)
        .map_err(|e| crate::error::Error::Message(e.to_string()))?;
    Ok(w.into_string())
}

#[derive(Debug)]
struct SerError {
    msg: String,
}

impl core::fmt::Display for SerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.msg)
    }
}
impl serde::ser::Error for SerError {
    fn custom<T: core::fmt::Display>(t: T) -> Self {
        SerError {
            msg: format!("{}", t),
        }
    }
}

impl core::error::Error for SerError {}

struct StreamingSerializer<'a> {
    w: &'a mut LineWriter,
    opts: &'a Options,
    indent: usize,
}

impl<'a> StreamingSerializer<'a> {
    fn with_indent<'b>(&'b mut self, indent: usize) -> StreamingSerializer<'b> {
        StreamingSerializer {
            w: self.w,
            opts: self.opts,
            indent,
        }
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

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.w.line(self.indent, primitives::format_bool(v));
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.w.line(self.indent, &v.to_string());
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.w.line(self.indent, &v.to_string());
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }
    fn serialize_f64(self, f: f64) -> Result<Self::Ok, Self::Error> {
        if f.is_finite() {
            self.w.line(self.indent, &primitives::format_f64(f));
        } else {
            self.w.line(self.indent, primitives::format_null());
        }
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.w.line(
            self.indent,
            &primitives::format_string(&v.to_string(), self.opts.delimiter),
        );
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.w.line(
            self.indent,
            &primitives::format_string(v, self.opts.delimiter),
        );
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for b in v {
            SerializeSeq::serialize_element(&mut seq, b)?;
        }
        SerializeSeq::end(seq)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.w.line(self.indent, primitives::format_null());
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.w.line(self.indent, primitives::format_null());
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.w.line_key_only(
            self.indent,
            &primitives::format_string(variant, self.opts.delimiter),
        );
        let mut child = self.with_indent(self.indent + self.opts.indent);
        value.serialize(&mut child)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        #[cfg(feature = "json")]
        {
            Ok(SeqSer {
                parent: self,
                items: Vec::new(),
            })
        }
        #[cfg(not(feature = "json"))]
        {
            Ok(SeqSerAlloc {
                parent: self,
                items: Vec::new(),
            })
        }
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_tuple_struct(variant, len)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSer {
            parent: self,
            next_key: None,
            entry_count: 0,
            #[cfg(feature = "json")]
            buffered: None,
        })
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(None)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.w.line_key_only(
            self.indent,
            &primitives::format_string(variant, self.opts.delimiter),
        );
        Ok(MapSer {
            parent: self,
            next_key: None,
            entry_count: 0,
            #[cfg(feature = "json")]
            buffered: None,
        })
    }
}

#[cfg(feature = "json")]
struct SeqSer<'a, 'de> {
    parent: &'a mut StreamingSerializer<'de>,
    items: Vec<Value>,
}

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
        let delim = self.parent.opts.delimiter;
        let dch = primitives::delimiter_char(delim);
        let len = self.items.len();

        if let Some(keys) = crate::encode::encoders::is_tabular_array(&self.items) {
            // Tabular: [N]{f1,f2,...}:
            let field_cells: Vec<String> = keys.iter().map(|k| primitives::format_key(k)).collect();
            let header = primitives::format_tabular_header(len, &field_cells, delim);
            self.parent.w.line(self.parent.indent, &header);

            // Rows at indent+2
            for item in &self.items {
                let obj = item.as_object().unwrap();
                let cells: Vec<String> = keys
                    .iter()
                    .map(|k| {
                        let v = obj.get(k).unwrap();
                        format_primitive_value_json(v, delim)
                    })
                    .collect();
                let row = join_with_delim(&cells, dch);
                self.parent
                    .w
                    .line(self.parent.indent + self.parent.opts.indent, &row);
            }
            Ok(())
        } else if self.items.is_empty() {
            // Empty array: [0]:
            self.parent.w.line(
                self.parent.indent,
                &primitives::format_expanded_array_header(0, delim),
            );
            Ok(())
        } else if is_primitive_array_json(&self.items) {
            // Inline primitive array: [N]: v1,v2,v3
            let values: Vec<String> = self
                .items
                .iter()
                .map(|v| format_primitive_value_json(v, delim))
                .collect();
            let inline = join_with_delim(&values, dch);
            self.parent.w.line(
                self.parent.indent,
                &format!(
                    "{}: {}",
                    primitives::format_bracket_segment(len, delim),
                    inline
                ),
            );
            Ok(())
        } else {
            // Mixed array: [N]: with list items
            self.parent.w.line(
                self.parent.indent,
                &primitives::format_expanded_array_header(len, delim),
            );
            for item in &self.items {
                encode_list_item_json(
                    item,
                    self.parent.w,
                    self.parent.opts,
                    self.parent.indent + self.parent.opts.indent,
                )?;
            }
            Ok(())
        }
    }
}

#[cfg(feature = "json")]
impl<'a, 'de> SerializeTuple for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

#[cfg(feature = "json")]
impl<'a, 'de> SerializeTupleStruct for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

#[cfg(feature = "json")]
impl<'a, 'de> SerializeTupleVariant for SeqSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

#[cfg(not(feature = "json"))]
struct SeqSerAlloc<'a, 'de> {
    parent: &'a mut StreamingSerializer<'de>,
    items: Vec<IValue>,
}

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
        let delim = self.parent.opts.delimiter;
        let dch = primitives::delimiter_char(delim);
        let len = self.items.len();

        if let Some(keys) = is_tabular_array_alloc(&self.items) {
            // Tabular: [N]{f1,f2,...}:
            let field_cells: Vec<String> = keys.iter().map(|k| primitives::format_key(k)).collect();
            let header = primitives::format_tabular_header(len, &field_cells, delim);
            self.parent.w.line(self.parent.indent, &header);

            // Rows at indent+2
            for item in &self.items {
                let obj = match item {
                    IValue::Object(pairs) => pairs,
                    _ => unreachable!(),
                };
                let cells: Vec<String> = keys
                    .iter()
                    .map(|k| {
                        let v = obj.iter().find(|(kk, _)| kk == k).map(|(_, v)| v).unwrap();
                        format_primitive_value_alloc(v, delim)
                    })
                    .collect();
                let row = join_with_delim(&cells, dch);
                self.parent
                    .w
                    .line(self.parent.indent + self.parent.opts.indent, &row);
            }
            Ok(())
        } else if self.items.is_empty() {
            // Empty array: [0]:
            self.parent.w.line(
                self.parent.indent,
                &primitives::format_expanded_array_header(0, delim),
            );
            Ok(())
        } else if is_primitive_array_alloc(&self.items) {
            // Inline primitive array: [N]: v1,v2,v3
            let values: Vec<String> = self
                .items
                .iter()
                .map(|v| format_primitive_value_alloc(v, delim))
                .collect();
            let inline = join_with_delim(&values, dch);
            self.parent.w.line(
                self.parent.indent,
                &format!(
                    "{}: {}",
                    primitives::format_bracket_segment(len, delim),
                    inline
                ),
            );
            Ok(())
        } else {
            // Mixed array: [N]: with list items
            self.parent.w.line(
                self.parent.indent,
                &primitives::format_expanded_array_header(len, delim),
            );
            for item in &self.items {
                encode_list_item_alloc(
                    item,
                    self.parent.w,
                    self.parent.opts,
                    self.parent.indent + self.parent.opts.indent,
                )?;
            }
            Ok(())
        }
    }
}

#[cfg(not(feature = "json"))]
impl<'a, 'de> SerializeTuple for SeqSerAlloc<'a, 'de> {
    type Ok = ();
    type Error = SerError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[cfg(not(feature = "json"))]
impl<'a, 'de> SerializeTupleStruct for SeqSerAlloc<'a, 'de> {
    type Ok = ();
    type Error = SerError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[cfg(not(feature = "json"))]
impl<'a, 'de> SerializeTupleVariant for SeqSerAlloc<'a, 'de> {
    type Ok = ();
    type Error = SerError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct MapSer<'a, 'de> {
    parent: &'a mut StreamingSerializer<'de>,
    next_key: Option<String>,
    entry_count: usize,
    /// Buffered entries for key folding collision detection
    #[cfg(feature = "json")]
    buffered: Option<Vec<(String, Value)>>,
}

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

        // When key folding is enabled, buffer entries for collision detection
        #[cfg(feature = "json")]
        if self.parent.opts.key_folding == crate::options::KeyFolding::Safe {
            let val = crate::ser::value_builder::to_value(value, self.parent.opts);
            if self.buffered.is_none() {
                self.buffered = Some(Vec::new());
            }
            self.buffered.as_mut().unwrap().push((key, val));
            self.entry_count += 1;
            return Ok(());
        }

        let key_fmt = primitives::format_key(&key);

        // Try scalar first
        if let Ok(sv) = try_scalar_to_string(value, self.parent.opts) {
            self.parent.w.line_kv(self.parent.indent, &key_fmt, &sv);
            self.entry_count += 1;
            return Ok(());
        }

        // Try to serialize to Value to detect arrays
        #[cfg(feature = "json")]
        {
            let val = crate::ser::value_builder::to_value(value, self.parent.opts);
            if let Value::Array(items) = val {
                // Use spec-compliant keyed array encoding
                encode_keyed_array_json(
                    &key_fmt,
                    &items,
                    self.parent.w,
                    self.parent.opts,
                    self.parent.indent,
                )?;
                self.entry_count += 1;
                return Ok(());
            }
            // For objects, use encode_object_field which handles key folding
            crate::encode::encoders::encode_object_field(
                &key,
                &val,
                self.parent.w,
                self.parent.opts,
                self.parent.indent,
            )
            .map_err(|e| SerError::custom(e.to_string()))?;
            self.entry_count += 1;
            Ok(())
        }

        #[cfg(not(feature = "json"))]
        {
            let val = crate::ser::value_builder_alloc::to_value(value, self.parent.opts);
            if let IValue::Array(items) = val {
                // Use spec-compliant keyed array encoding
                encode_keyed_array_alloc(
                    &key_fmt,
                    &items,
                    self.parent.w,
                    self.parent.opts,
                    self.parent.indent,
                )?;
                self.entry_count += 1;
                return Ok(());
            }
            // For objects, use the standard key: then nested fields
            self.parent.w.line_key_only(self.parent.indent, &key_fmt);
            encode_internal_value_alloc(
                &val,
                self.parent.w,
                self.parent.opts,
                self.parent.indent + self.parent.opts.indent,
            )?;
            self.entry_count += 1;
            Ok(())
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Emit buffered entries with collision detection
        #[cfg(feature = "json")]
        if let Some(entries) = self.buffered {
            // Collect all keys for collision detection
            let sibling_keys: Vec<String> = entries.iter().map(|(k, _)| k.clone()).collect();

            for (key, val) in entries {
                crate::encode::encoders::encode_object_field_with_siblings(
                    &key,
                    &val,
                    self.parent.w,
                    self.parent.opts,
                    self.parent.indent,
                    &sibling_keys,
                )
                .map_err(|e| SerError::custom(e.to_string()))?;
            }
        }
        Ok(())
    }
}

impl<'a, 'de> SerializeStruct for MapSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeMap::serialize_key(self, &key)?;
        SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'de> SerializeStructVariant for MapSer<'a, 'de> {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeStruct::end(self)
    }
}

fn join_with_delim(cells: &[String], dch: char) -> String {
    cells.join(&dch.to_string())
}

#[cfg(feature = "json")]
fn is_primitive_array_json(items: &[Value]) -> bool {
    items.iter().all(|v| {
        matches!(
            v,
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
        )
    })
}

#[cfg(feature = "json")]
fn format_primitive_value_json(v: &Value, delim: crate::options::Delimiter) -> String {
    match v {
        Value::Null => primitives::format_null().to_string(),
        Value::Bool(b) => primitives::format_bool(*b).to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => primitives::format_string(s, delim),
        _ => "null".to_string(),
    }
}

#[cfg(feature = "json")]
fn encode_list_item_json(
    item: &Value,
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<(), SerError> {
    match item {
        Value::Null => w.line_list_item(indent, primitives::format_null()),
        Value::Bool(b) => w.line_list_item(indent, primitives::format_bool(*b)),
        Value::Number(n) => w.line_list_item(indent, &n.to_string()),
        Value::String(s) => w.line_list_item(indent, &primitives::format_string(s, opts.delimiter)),
        Value::Array(items) => {
            let len = items.len();
            let delim = opts.delimiter;
            let dch = primitives::delimiter_char(delim);
            if items.is_empty() {
                w.line_list_item(
                    indent,
                    &format!("{}:", primitives::format_bracket_segment(0, delim)),
                );
            } else if is_primitive_array_json(items) {
                let values: Vec<String> = items
                    .iter()
                    .map(|v| format_primitive_value_json(v, delim))
                    .collect();
                let inline = join_with_delim(&values, dch);
                w.line_list_item(
                    indent,
                    &format!(
                        "{}: {}",
                        primitives::format_bracket_segment(len, delim),
                        inline
                    ),
                );
            } else {
                w.line_list_item(
                    indent,
                    &primitives::format_expanded_array_header(len, delim),
                );
                for inner in items {
                    encode_list_item_json(inner, w, opts, indent + opts.indent)?;
                }
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                w.line(indent, "-");
                return Ok(());
            }
            let mut iter = obj.iter();
            let (first_key, first_value) = iter.next().unwrap();
            let first_key_fmt = primitives::format_key(first_key);

            // Special case: first field is a tabular array (§10)
            if let Value::Array(items) = first_value {
                if let Some(keys) = crate::encode::encoders::is_tabular_array(items) {
                    let delim = opts.delimiter;
                    let dch = primitives::delimiter_char(delim);
                    let field_cells: Vec<String> =
                        keys.iter().map(|k| primitives::format_key(k)).collect();
                    let header = format!(
                        "{}{}",
                        first_key_fmt,
                        primitives::format_tabular_header(items.len(), &field_cells, delim)
                    );
                    w.line_list_item(indent, &header);

                    // Rows at indent + 4
                    for item in items {
                        let inner_obj = item.as_object().unwrap();
                        let cells: Vec<String> = keys
                            .iter()
                            .map(|k| {
                                let v = inner_obj.get(k).unwrap();
                                format_primitive_value_json(v, delim)
                            })
                            .collect();
                        let row = join_with_delim(&cells, dch);
                        w.line(indent + 4, &row);
                    }

                    // Remaining fields at depth + 1 (indent + opts.indent)
                    for (k, v) in iter {
                        let key_fmt = primitives::format_key(k);
                        match v {
                            Value::Null => {
                                w.line_kv(indent + opts.indent, &key_fmt, primitives::format_null())
                            }
                            Value::Bool(b) => w.line_kv(
                                indent + opts.indent,
                                &key_fmt,
                                primitives::format_bool(*b),
                            ),
                            Value::Number(n) => {
                                w.line_kv(indent + opts.indent, &key_fmt, &n.to_string())
                            }
                            Value::String(s) => w.line_kv(
                                indent + opts.indent,
                                &key_fmt,
                                &primitives::format_string(s, opts.delimiter),
                            ),
                            Value::Array(arr) => {
                                encode_keyed_array_json(
                                    &key_fmt,
                                    arr,
                                    w,
                                    opts,
                                    indent + opts.indent,
                                )?;
                            }
                            Value::Object(_) => {
                                w.line_key_only(indent + opts.indent, &key_fmt);
                                crate::encode::encoders::encode_value(v, w, opts, indent + 4)
                                    .map_err(|e| SerError::custom(e.to_string()))?;
                            }
                        }
                    }
                    return Ok(());
                }
            }

            // Standard case: first field on hyphen line
            match first_value {
                Value::Null => w.line(
                    indent,
                    &format!("- {}: {}", first_key_fmt, primitives::format_null()),
                ),
                Value::Bool(b) => w.line(
                    indent,
                    &format!("- {}: {}", first_key_fmt, primitives::format_bool(*b)),
                ),
                Value::Number(n) => w.line(indent, &format!("- {}: {}", first_key_fmt, n)),
                Value::String(s) => {
                    let v = primitives::format_string(s, opts.delimiter);
                    w.line(indent, &format!("- {}: {}", first_key_fmt, v));
                }
                Value::Array(items) => {
                    // Non-tabular array as first field
                    let len = items.len();
                    let delim = opts.delimiter;
                    if items.is_empty() {
                        w.line(
                            indent,
                            &format!(
                                "- {}{}:",
                                first_key_fmt,
                                primitives::format_bracket_segment(0, delim)
                            ),
                        );
                    } else if is_primitive_array_json(items) {
                        let dch = primitives::delimiter_char(delim);
                        let values: Vec<String> = items
                            .iter()
                            .map(|v| format_primitive_value_json(v, delim))
                            .collect();
                        let inline = join_with_delim(&values, dch);
                        w.line(
                            indent,
                            &format!(
                                "- {}{}: {}",
                                first_key_fmt,
                                primitives::format_bracket_segment(len, delim),
                                inline
                            ),
                        );
                    } else {
                        w.line(
                            indent,
                            &format!(
                                "- {}{}",
                                first_key_fmt,
                                primitives::format_expanded_array_header(len, delim)
                            ),
                        );
                        for inner in items {
                            encode_list_item_json(inner, w, opts, indent + 4)?;
                        }
                    }
                }
                Value::Object(inner_obj) => {
                    w.line(indent, &format!("- {}:", first_key_fmt));
                    if !inner_obj.is_empty() {
                        for (k, v) in inner_obj {
                            let key_fmt = primitives::format_key(k);
                            match v {
                                Value::Null => {
                                    w.line_kv(indent + 4, &key_fmt, primitives::format_null())
                                }
                                Value::Bool(b) => {
                                    w.line_kv(indent + 4, &key_fmt, primitives::format_bool(*b))
                                }
                                Value::Number(n) => w.line_kv(indent + 4, &key_fmt, &n.to_string()),
                                Value::String(s) => w.line_kv(
                                    indent + 4,
                                    &key_fmt,
                                    &primitives::format_string(s, opts.delimiter),
                                ),
                                Value::Array(arr) => {
                                    encode_keyed_array_json(&key_fmt, arr, w, opts, indent + 4)?;
                                }
                                Value::Object(_) => {
                                    w.line_key_only(indent + 4, &key_fmt);
                                    crate::encode::encoders::encode_value(v, w, opts, indent + 6)
                                        .map_err(|e| SerError::custom(e.to_string()))?;
                                }
                            }
                        }
                    }
                }
            }

            // Remaining fields at depth + 1 (indent + opts.indent)
            for (k, v) in iter {
                let key_fmt = primitives::format_key(k);
                match v {
                    Value::Null => {
                        w.line_kv(indent + opts.indent, &key_fmt, primitives::format_null())
                    }
                    Value::Bool(b) => {
                        w.line_kv(indent + opts.indent, &key_fmt, primitives::format_bool(*b))
                    }
                    Value::Number(n) => w.line_kv(indent + opts.indent, &key_fmt, &n.to_string()),
                    Value::String(s) => w.line_kv(
                        indent + opts.indent,
                        &key_fmt,
                        &primitives::format_string(s, opts.delimiter),
                    ),
                    Value::Array(arr) => {
                        encode_keyed_array_json(&key_fmt, arr, w, opts, indent + opts.indent)?;
                    }
                    Value::Object(_) => {
                        w.line_key_only(indent + opts.indent, &key_fmt);
                        crate::encode::encoders::encode_value(v, w, opts, indent + 4)
                            .map_err(|e| SerError::custom(e.to_string()))?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Encode a keyed array with spec-compliant format: key[N]: v1,v2 or key[N]{fields}:
#[cfg(feature = "json")]
fn encode_keyed_array_json(
    key: &str,
    items: &[Value],
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<(), SerError> {
    let len = items.len();
    let delim = opts.delimiter;
    let dch = primitives::delimiter_char(delim);

    if items.is_empty() {
        // Empty array: key[0]:
        w.line(
            indent,
            &format!("{}{}:", key, primitives::format_bracket_segment(0, delim)),
        );
        return Ok(());
    }

    // Check for tabular array
    if let Some(keys) = crate::encode::encoders::is_tabular_array(items) {
        let field_cells: Vec<String> = keys.iter().map(|k| primitives::format_key(k)).collect();
        let header = format!(
            "{}{}",
            key,
            primitives::format_tabular_header(len, &field_cells, delim)
        );
        w.line(indent, &header);

        // Rows at indent+2
        for item in items {
            let obj = item.as_object().unwrap();
            let cells: Vec<String> = keys
                .iter()
                .map(|k| {
                    let v = obj.get(k).unwrap();
                    format_primitive_value_json(v, delim)
                })
                .collect();
            let row = join_with_delim(&cells, dch);
            w.line(indent + opts.indent, &row);
        }
        return Ok(());
    }

    // Check for inline primitive array
    if is_primitive_array_json(items) {
        let values: Vec<String> = items
            .iter()
            .map(|v| format_primitive_value_json(v, delim))
            .collect();
        let inline = join_with_delim(&values, dch);
        w.line(
            indent,
            &format!(
                "{}{}: {}",
                key,
                primitives::format_bracket_segment(len, delim),
                inline
            ),
        );
        return Ok(());
    }

    // Mixed/complex array: key[N]: with list items
    w.line(
        indent,
        &format!(
            "{}{}",
            key,
            primitives::format_expanded_array_header(len, delim)
        ),
    );
    for item in items {
        encode_list_item_json(item, w, opts, indent + opts.indent)?;
    }
    Ok(())
}

#[cfg(not(feature = "json"))]
fn is_primitive_array_alloc(items: &[IValue]) -> bool {
    items.iter().all(|v| v.is_primitive())
}

#[cfg(not(feature = "json"))]
fn format_primitive_value_alloc(v: &IValue, delim: crate::options::Delimiter) -> String {
    match v {
        IValue::Null => primitives::format_null().to_string(),
        IValue::Bool(b) => primitives::format_bool(*b).to_string(),
        IValue::Number(n) => n.to_string(),
        IValue::String(s) => primitives::format_string(s, delim),
        _ => "null".to_string(),
    }
}

#[cfg(not(feature = "json"))]
fn encode_list_item_alloc(
    item: &IValue,
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<(), SerError> {
    match item {
        IValue::Null => w.line_list_item(indent, primitives::format_null()),
        IValue::Bool(b) => w.line_list_item(indent, primitives::format_bool(*b)),
        IValue::Number(n) => w.line_list_item(indent, &n.to_string()),
        IValue::String(s) => {
            w.line_list_item(indent, &primitives::format_string(s, opts.delimiter))
        }
        IValue::Array(items) => {
            let len = items.len();
            let delim = opts.delimiter;
            let dch = primitives::delimiter_char(delim);
            if items.is_empty() {
                w.line_list_item(
                    indent,
                    &format!("{}:", primitives::format_bracket_segment(0, delim)),
                );
            } else if is_primitive_array_alloc(items) {
                let values: Vec<String> = items
                    .iter()
                    .map(|v| format_primitive_value_alloc(v, delim))
                    .collect();
                let inline = join_with_delim(&values, dch);
                w.line_list_item(
                    indent,
                    &format!(
                        "{}: {}",
                        primitives::format_bracket_segment(len, delim),
                        inline
                    ),
                );
            } else {
                w.line_list_item(
                    indent,
                    &primitives::format_expanded_array_header(len, delim),
                );
                for inner in items {
                    encode_list_item_alloc(inner, w, opts, indent + opts.indent)?;
                }
            }
        }
        IValue::Object(pairs) => {
            if pairs.is_empty() {
                w.line(indent, "-");
                return Ok(());
            }
            let mut iter = pairs.iter();
            let (first_key, first_value) = iter.next().unwrap();
            let first_key_fmt = primitives::format_key(first_key);

            // Special case: first field is a tabular array (§10)
            if let IValue::Array(items) = first_value {
                if let Some(keys) = is_tabular_array_alloc(items) {
                    let delim = opts.delimiter;
                    let dch = primitives::delimiter_char(delim);
                    let field_cells: Vec<String> =
                        keys.iter().map(|k| primitives::format_key(k)).collect();
                    let header = format!(
                        "{}{}",
                        first_key_fmt,
                        primitives::format_tabular_header(items.len(), &field_cells, delim)
                    );
                    w.line_list_item(indent, &header);

                    // Rows at indent + 4
                    for item in items {
                        let obj = match item {
                            IValue::Object(p) => p,
                            _ => unreachable!(),
                        };
                        let cells: Vec<String> = keys
                            .iter()
                            .map(|k| {
                                let v = obj.iter().find(|(kk, _)| kk == k).map(|(_, v)| v).unwrap();
                                format_primitive_value_alloc(v, delim)
                            })
                            .collect();
                        let row = join_with_delim(&cells, dch);
                        w.line(indent + 4, &row);
                    }

                    // Remaining fields at depth + 1 (indent + opts.indent)
                    for (k, v) in iter {
                        let key_fmt = primitives::format_key(k);
                        match v {
                            IValue::Null => {
                                w.line_kv(indent + opts.indent, &key_fmt, primitives::format_null())
                            }
                            IValue::Bool(b) => w.line_kv(
                                indent + opts.indent,
                                &key_fmt,
                                primitives::format_bool(*b),
                            ),
                            IValue::Number(n) => {
                                w.line_kv(indent + opts.indent, &key_fmt, &n.to_string())
                            }
                            IValue::String(s) => w.line_kv(
                                indent + opts.indent,
                                &key_fmt,
                                &primitives::format_string(s, opts.delimiter),
                            ),
                            IValue::Array(arr) => {
                                encode_keyed_array_alloc(
                                    &key_fmt,
                                    arr,
                                    w,
                                    opts,
                                    indent + opts.indent,
                                )?;
                            }
                            IValue::Object(_) => {
                                w.line_key_only(indent + opts.indent, &key_fmt);
                                encode_internal_value_alloc(v, w, opts, indent + 4)?;
                            }
                        }
                    }
                    return Ok(());
                }
            }

            // Standard case: first field on hyphen line
            match first_value {
                IValue::Null => w.line(
                    indent,
                    &format!("- {}: {}", first_key_fmt, primitives::format_null()),
                ),
                IValue::Bool(b) => w.line(
                    indent,
                    &format!("- {}: {}", first_key_fmt, primitives::format_bool(*b)),
                ),
                IValue::Number(n) => w.line(indent, &format!("- {}: {}", first_key_fmt, n)),
                IValue::String(s) => {
                    let v = primitives::format_string(s, opts.delimiter);
                    w.line(indent, &format!("- {}: {}", first_key_fmt, v));
                }
                IValue::Array(items) => {
                    // Non-tabular array as first field
                    let len = items.len();
                    let delim = opts.delimiter;
                    if items.is_empty() {
                        w.line(
                            indent,
                            &format!(
                                "- {}{}:",
                                first_key_fmt,
                                primitives::format_bracket_segment(0, delim)
                            ),
                        );
                    } else if is_primitive_array_alloc(items) {
                        let dch = primitives::delimiter_char(delim);
                        let values: Vec<String> = items
                            .iter()
                            .map(|v| format_primitive_value_alloc(v, delim))
                            .collect();
                        let inline = join_with_delim(&values, dch);
                        w.line(
                            indent,
                            &format!(
                                "- {}{}: {}",
                                first_key_fmt,
                                primitives::format_bracket_segment(len, delim),
                                inline
                            ),
                        );
                    } else {
                        w.line(
                            indent,
                            &format!(
                                "- {}{}",
                                first_key_fmt,
                                primitives::format_expanded_array_header(len, delim)
                            ),
                        );
                        for inner in items {
                            encode_list_item_alloc(inner, w, opts, indent + 4)?;
                        }
                    }
                }
                IValue::Object(inner_obj) => {
                    w.line(indent, &format!("- {}:", first_key_fmt));
                    if !inner_obj.is_empty() {
                        for (k, v) in inner_obj {
                            let key_fmt = primitives::format_key(k);
                            match v {
                                IValue::Null => {
                                    w.line_kv(indent + 4, &key_fmt, primitives::format_null())
                                }
                                IValue::Bool(b) => {
                                    w.line_kv(indent + 4, &key_fmt, primitives::format_bool(*b))
                                }
                                IValue::Number(n) => {
                                    w.line_kv(indent + 4, &key_fmt, &n.to_string())
                                }
                                IValue::String(s) => w.line_kv(
                                    indent + 4,
                                    &key_fmt,
                                    &primitives::format_string(s, opts.delimiter),
                                ),
                                IValue::Array(arr) => {
                                    encode_keyed_array_alloc(&key_fmt, arr, w, opts, indent + 4)?;
                                }
                                IValue::Object(_) => {
                                    w.line_key_only(indent + 4, &key_fmt);
                                    encode_internal_value_alloc(v, w, opts, indent + 6)?;
                                }
                            }
                        }
                    }
                }
            }

            // Remaining fields at depth + 1 (indent + opts.indent)
            for (k, v) in iter {
                let key_fmt = primitives::format_key(k);
                match v {
                    IValue::Null => {
                        w.line_kv(indent + opts.indent, &key_fmt, primitives::format_null())
                    }
                    IValue::Bool(b) => {
                        w.line_kv(indent + opts.indent, &key_fmt, primitives::format_bool(*b))
                    }
                    IValue::Number(n) => w.line_kv(indent + opts.indent, &key_fmt, &n.to_string()),
                    IValue::String(s) => w.line_kv(
                        indent + opts.indent,
                        &key_fmt,
                        &primitives::format_string(s, opts.delimiter),
                    ),
                    IValue::Array(arr) => {
                        encode_keyed_array_alloc(&key_fmt, arr, w, opts, indent + opts.indent)?;
                    }
                    IValue::Object(_) => {
                        w.line_key_only(indent + opts.indent, &key_fmt);
                        encode_internal_value_alloc(v, w, opts, indent + 4)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Encode a keyed array with spec-compliant format (alloc version)
#[cfg(not(feature = "json"))]
fn encode_keyed_array_alloc(
    key: &str,
    items: &[IValue],
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<(), SerError> {
    let len = items.len();
    let delim = opts.delimiter;
    let dch = primitives::delimiter_char(delim);

    if items.is_empty() {
        w.line(
            indent,
            &format!("{}{}:", key, primitives::format_bracket_segment(0, delim)),
        );
        return Ok(());
    }

    // Check for tabular array
    if let Some(keys) = is_tabular_array_alloc(items) {
        let field_cells: Vec<String> = keys.iter().map(|k| primitives::format_key(k)).collect();
        let header = format!(
            "{}{}",
            key,
            primitives::format_tabular_header(len, &field_cells, delim)
        );
        w.line(indent, &header);

        for item in items {
            let obj = match item {
                IValue::Object(pairs) => pairs,
                _ => unreachable!(),
            };
            let cells: Vec<String> = keys
                .iter()
                .map(|k| {
                    let v = obj.iter().find(|(kk, _)| kk == k).map(|(_, v)| v).unwrap();
                    format_primitive_value_alloc(v, delim)
                })
                .collect();
            let row = join_with_delim(&cells, dch);
            w.line(indent + opts.indent, &row);
        }
        return Ok(());
    }

    // Check for inline primitive array
    if is_primitive_array_alloc(items) {
        let values: Vec<String> = items
            .iter()
            .map(|v| format_primitive_value_alloc(v, delim))
            .collect();
        let inline = join_with_delim(&values, dch);
        w.line(
            indent,
            &format!(
                "{}{}: {}",
                key,
                primitives::format_bracket_segment(len, delim),
                inline
            ),
        );
        return Ok(());
    }

    // Mixed array
    w.line(
        indent,
        &format!(
            "{}{}",
            key,
            primitives::format_expanded_array_header(len, delim)
        ),
    );
    for item in items {
        encode_list_item_alloc(item, w, opts, indent + opts.indent)?;
    }
    Ok(())
}

// --- helpers for non-json path ---
#[cfg(not(feature = "json"))]
fn is_tabular_array_alloc(arr: &[IValue]) -> Option<Vec<String>> {
    if arr.is_empty() {
        return None;
    }
    let mut keys: Option<Vec<String>> = None;
    for v in arr {
        let obj = match v {
            IValue::Object(pairs) => pairs,
            _ => return None,
        };
        let mut kset: Vec<String> = obj.iter().map(|(k, _)| k.clone()).collect();
        kset.sort();
        if let Some(ref ks) = keys {
            if *ks != kset {
                return None;
            }
        } else {
            keys = Some(kset);
        }
        for (_, vv) in obj.iter() {
            if !vv.is_primitive() {
                return None;
            }
        }
    }
    keys
}

#[cfg(not(feature = "json"))]
fn encode_internal_value_alloc(
    v: &IValue,
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<(), SerError> {
    match v {
        IValue::Null => {
            w.line(indent, primitives::format_null());
        }
        IValue::Bool(b) => {
            w.line(indent, primitives::format_bool(*b));
        }
        IValue::Number(n) => {
            w.line(indent, &n.to_string());
        }
        IValue::String(s) => {
            w.line(indent, &primitives::format_string(s, opts.delimiter));
        }
        IValue::Array(items) => {
            for item in items {
                if item.is_primitive() {
                    match item {
                        IValue::Null => w.line_list_item(indent, primitives::format_null()),
                        IValue::Bool(b) => w.line_list_item(indent, primitives::format_bool(*b)),
                        IValue::Number(n) => w.line_list_item(indent, &n.to_string()),
                        IValue::String(s) => {
                            w.line_list_item(indent, &primitives::format_string(s, opts.delimiter))
                        }
                        _ => {}
                    }
                } else {
                    w.line(indent, "-");
                    encode_internal_value_alloc(item, w, opts, indent + opts.indent)?;
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
                        IValue::String(s) => {
                            w.line_kv(indent, &key, &primitives::format_string(s, opts.delimiter))
                        }
                        _ => {}
                    }
                } else {
                    w.line_key_only(indent, &key);
                    encode_internal_value_alloc(val, w, opts, indent + opts.indent)?;
                }
            }
        }
    }
    Ok(())
}

fn try_scalar_to_string<T: ?Sized + Serialize>(
    value: &T,
    opts: &Options,
) -> Result<String, SerError> {
    let mut s = ScalarSerializer { out: None, opts };
    match value.serialize(&mut s) {
        Ok(()) => s.out.ok_or_else(|| SerError::custom("no scalar")),
        Err(e) => Err(e),
    }
}

struct ScalarSerializer<'a> {
    out: Option<String>,
    opts: &'a Options,
}

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

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.out = Some(primitives::format_bool(v).to_string());
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        if v.is_finite() {
            self.out = Some(primitives::format_f64(v));
        } else if v.is_nan() {
            self.out = Some(primitives::escape_and_quote("NaN"));
        } else if v.is_sign_positive() {
            self.out = Some(primitives::escape_and_quote("Infinity"));
        } else {
            self.out = Some(primitives::escape_and_quote("-Infinity"));
        }
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.out = Some(primitives::format_string(
            &v.to_string(),
            self.opts.delimiter,
        ));
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.out = Some(primitives::format_string(v, self.opts.delimiter));
        Ok(())
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.out = Some(primitives::format_null().to_string());
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.out = Some(primitives::format_null().to_string());
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerError::custom("non-scalar"))
    }
}

fn key_to_string<T: ?Sized + Serialize>(key: &T, _opts: &Options) -> Result<String, SerError> {
    let mut s = KeySerializer { out: None };
    key.serialize(&mut s)?;
    s.out.ok_or_else(|| SerError::custom("invalid key"))
}

struct KeySerializer {
    out: Option<String>,
}

impl Serializer for &mut KeySerializer {
    type Ok = ();
    type Error = SerError;
    type SerializeSeq = Impossible<(), SerError>;
    type SerializeTuple = Impossible<(), SerError>;
    type SerializeTupleStruct = Impossible<(), SerError>;
    type SerializeTupleVariant = Impossible<(), SerError>;
    type SerializeMap = Impossible<(), SerError>;
    type SerializeStruct = Impossible<(), SerError>;
    type SerializeStructVariant = Impossible<(), SerError>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.out = Some(if v { "true".into() } else { "false".into() });
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.out = Some(primitives::format_f64(v as f64));
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.out = Some(primitives::format_f64(v));
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.out = Some(v.to_string());
        Ok(())
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.out = Some("null".into());
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.out = Some("null".into());
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerError::custom("invalid key"))
    }
}
