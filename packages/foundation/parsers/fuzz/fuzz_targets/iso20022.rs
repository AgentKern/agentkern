#![no_main]
use libfuzzer_sys::fuzz_target;
use agentkern_connectors::swift::mx_parser::MxParser;

fuzz_target!(|data: &[u8]| {
    if let Ok(xml) = std::str::from_utf8(data) {
        // Attempt to parse the fuzzed input
        // The parser should treat invalid input gracefully and return an error,
        // but it should NEVER panic.
        let parser = MxParser::new();
        let _ = parser.parse(xml);
    }
});
