#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Mirror what `csv to-json` does.
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(data);
    let _ = reader.headers().cloned();
    for record in reader.records() {
        let _ = record;
    }
});
