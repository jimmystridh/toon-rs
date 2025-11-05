#![no_main]
use libfuzzer_sys::fuzz_target;
use toon::{Options, decode_from_str};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let opts = Options::default();
        let _ = decode_from_str::<serde_json::Value>(s, &opts);
    }
});
