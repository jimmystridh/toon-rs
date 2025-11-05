#![no_main]
use libfuzzer_sys::fuzz_target;
use toon::{Options, decode_from_str, encode_to_string};

fuzz_target!(|data: &[u8]| {
    if let Ok(toon_input) = std::str::from_utf8(data) {
        let opts = Options::default();

        if let Ok(first_decode) = decode_from_str::<serde_json::Value>(toon_input, &opts) {
            if let Ok(encoded) = encode_to_string(&first_decode, &opts) {
                match decode_from_str::<serde_json::Value>(&encoded, &opts) {
                    Ok(second_decode) => {
                        if first_decode != second_decode {
                            panic!(
                                "TOON roundtrip mismatch!\nInput TOON: {}\nFirst decode: {}\nRe-encoded: {}\nSecond decode: {}",
                                toon_input,
                                serde_json::to_string_pretty(&first_decode).unwrap(),
                                encoded,
                                serde_json::to_string_pretty(&second_decode).unwrap()
                            );
                        }
                    }
                    Err(e) => {
                        panic!(
                            "Failed to decode re-encoded TOON!\nInput: {}\nFirst decode: {}\nRe-encoded: {}\nError: {}",
                            toon_input,
                            serde_json::to_string_pretty(&first_decode).unwrap(),
                            encoded,
                            e
                        );
                    }
                }
            }
        }
    }
});
