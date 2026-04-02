#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Treat the raw bytes as a UTF-8 string (lossy) and feed it to the TOML
    // scene loader.  We only care that the function never panics or triggers
    // undefined behavior — parse errors are fine.
    let input = std::str::from_utf8(data).unwrap_or_default();
    let _ = kiran::scene::load_scene(input);
});
