#![no_main]
use libfuzzer_sys::fuzz_target;
use toon::{Options, encode_to_string};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(s) {
            let opts = Options::default();
            let _ = encode_to_string(&json_value, &opts);

            let mut opts_pipe = Options::default();
            opts_pipe.delimiter = toon::Delimiter::Pipe;
            let _ = encode_to_string(&json_value, &opts_pipe);

            let mut opts_comma = Options::default();
            opts_comma.delimiter = toon::Delimiter::Comma;
            let _ = encode_to_string(&json_value, &opts_comma);
        }
    }
});
