#![no_main]
use libfuzzer_sys::fuzz_target;
use agentkern_parsers::idoc::IDocParser;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let parser = IDocParser::new();
        // IDOC parser expects line-based input, so generic random bytes might just fail early,
        // which is fine. We want to ensure no panics on weird input.
        let _ = parser.parse(text);
        
        // Also try XML parsing path
        let _ = parser.parse_xml(text);
    }
});
