#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    // Treat the raw bytes as a UTF-8 path string (lossy).  The function must
    // never panic — it should return Some or None gracefully.
    let input = std::str::from_utf8(data).unwrap_or_default();
    let _ = kiran::asset::validate_asset_path(Path::new(input));
});
