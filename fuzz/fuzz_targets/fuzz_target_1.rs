
#![no_main]
use libfuzzer_sys::fuzz_target;
use chinenshichanaka::convert;

fuzz_target!(|data: &[u8]| {
    // Try to decode the fuzzed data as an image
    if let Ok(img) = image::load_from_memory(data) {
        // Call the convert function with the decoded image
        let _ = convert(img);
    }
});
