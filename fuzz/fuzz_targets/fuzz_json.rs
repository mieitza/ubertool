#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Errors are fine; panics are bugs.
        let _ = ubertool::commands::json::convert::parse_json(s);
    }
});
