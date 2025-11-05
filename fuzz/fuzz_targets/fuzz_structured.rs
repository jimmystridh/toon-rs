#![no_main]
use libfuzzer_sys::{arbitrary, fuzz_target};
use toon::{Options, decode_from_str, encode_to_string};
use serde_json::{Value, Number};
use arbitrary::Arbitrary;

const MAX_DEPTH: usize = 8;
const MAX_ARRAY_SIZE: usize = 20;
const MAX_OBJECT_SIZE: usize = 20;

#[derive(Arbitrary, Debug)]
struct FuzzValue {
    choice: u8,
}

impl FuzzValue {
    fn to_json_value(&self, u: &mut arbitrary::Unstructured, depth: usize) -> arbitrary::Result<Value> {
        if depth >= MAX_DEPTH {
            return Ok(Value::Null);
        }

        Ok(match self.choice % 10 {
            0 => Value::Null,
            1 => Value::Bool(u.arbitrary()?),
            2 => {
                let n: i64 = u.arbitrary()?;
                Value::Number(Number::from(n))
            }
            3 => {
                let n: f64 = u.arbitrary()?;
                if n.is_finite() {
                    serde_json::json!(n)
                } else {
                    Value::Null
                }
            }
            4 => {
                let s: String = u.arbitrary()?;
                Value::String(s)
            }
            5..=7 => {
                let size = u.int_in_range(0..=MAX_ARRAY_SIZE)?;
                let mut arr = Vec::with_capacity(size);
                for _ in 0..size {
                    let fv: FuzzValue = u.arbitrary()?;
                    arr.push(fv.to_json_value(u, depth + 1)?);
                }
                Value::Array(arr)
            }
            _ => {
                let size = u.int_in_range(0..=MAX_OBJECT_SIZE)?;
                let mut obj = serde_json::Map::new();
                for _ in 0..size {
                    let key: String = u.arbitrary()?;
                    let fv: FuzzValue = u.arbitrary()?;
                    obj.insert(key, fv.to_json_value(u, depth + 1)?);
                }
                Value::Object(obj)
            }
        })
    }
}

fuzz_target!(|data: &[u8]| {
    let mut u = arbitrary::Unstructured::new(data);

    if let Ok(fv) = u.arbitrary::<FuzzValue>() {
        if let Ok(value) = fv.to_json_value(&mut u, 0) {
            let opts = Options::default();

            if let Ok(toon_str) = encode_to_string(&value, &opts) {
                match decode_from_str::<serde_json::Value>(&toon_str, &opts) {
                    Ok(decoded) => {
                        if value != decoded {
                            panic!(
                                "Structured roundtrip mismatch!\nOriginal: {}\nTOON: {}\nDecoded: {}",
                                serde_json::to_string_pretty(&value).unwrap(),
                                toon_str,
                                serde_json::to_string_pretty(&decoded).unwrap()
                            );
                        }
                    }
                    Err(e) => {
                        panic!(
                            "Failed to decode structured input!\nOriginal: {}\nTOON: {}\nError: {}",
                            serde_json::to_string_pretty(&value).unwrap(),
                            toon_str,
                            e
                        );
                    }
                }
            }
        }
    }
});
