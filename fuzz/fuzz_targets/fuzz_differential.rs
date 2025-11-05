#![no_main]
use libfuzzer_sys::fuzz_target;
use toon::{Options, decode_from_str};

fuzz_target!(|data: &[u8]| {
    if let Ok(toon_str) = std::str::from_utf8(data) {
        let opts_default = Options::default();
        let opts_strict = Options { strict: true, ..Options::default() };

        let result_default = decode_from_str::<serde_json::Value>(toon_str, &opts_default);
        let result_strict = decode_from_str::<serde_json::Value>(toon_str, &opts_strict);

        match (&result_default, &result_strict) {
            (Ok(val_default), Ok(val_strict)) => {
                if val_default != val_strict {
                    panic!(
                        "Differential fuzzing: different values!\nInput: {}\nDefault: {}\nStrict: {}",
                        toon_str,
                        serde_json::to_string_pretty(val_default).unwrap(),
                        serde_json::to_string_pretty(val_strict).unwrap()
                    );
                }
            }
            (Ok(_), Err(_)) => {
            }
            (Err(_), Ok(_)) => {
                panic!(
                    "Differential fuzzing: strict succeeded but default failed!\nInput: {}",
                    toon_str
                );
            }
            (Err(_), Err(_)) => {
            }
        }
    }
});
