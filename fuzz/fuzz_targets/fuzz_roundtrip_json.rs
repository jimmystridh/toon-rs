#![no_main]
use libfuzzer_sys::fuzz_target;
use toon::{Options, decode_from_str, encode_to_string};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(original_value) = serde_json::from_str::<serde_json::Value>(s) {
            let opts = Options::default();

            if let Ok(toon_str) = encode_to_string(&original_value, &opts) {
                match decode_from_str::<serde_json::Value>(&toon_str, &opts) {
                    Ok(decoded_value) => {
                        if original_value != decoded_value {
                            panic!(
                                "Roundtrip mismatch!\nOriginal JSON: {}\nTOON: {}\nDecoded: {}",
                                serde_json::to_string_pretty(&original_value).unwrap(),
                                toon_str,
                                serde_json::to_string_pretty(&decoded_value).unwrap()
                            );
                        }
                    }
                    Err(e) => {
                        panic!(
                            "Failed to decode valid TOON!\nOriginal: {}\nTOON: {}\nError: {}",
                            serde_json::to_string_pretty(&original_value).unwrap(),
                            toon_str,
                            e
                        );
                    }
                }
            }
        }
    }
});
