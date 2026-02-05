#![no_main]
use libfuzzer_sys::fuzz_target;
use agentkern_parsers::swift_mt::SwiftMtParser;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let parser = SwiftMtParser::new();
        let _ = parser.parse(text);
    }
});
