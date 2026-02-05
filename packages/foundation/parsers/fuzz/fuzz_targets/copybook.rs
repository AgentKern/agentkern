#![no_main]
use libfuzzer_sys::fuzz_target;
use agentkern_parsers::copybook::CopybookParser;

fuzz_target!(|data: &[u8]| {
    // Fuzzing the data parsing against a fixed copybook
    // This simulates receiving bad binary data from a mainframe
    
    // A simple sample copybook for testing data resilience
    const SAMPLE_COPYBOOK: &str = "
       01  RECORD.
           05  ID       PIC 9(5).
           05  NAME     PIC X(10).
           05  VAL      PIC S9(5)V99.
    ";

    if let Ok(parser) = CopybookParser::new(SAMPLE_COPYBOOK) {
        let _ = parser.parse_record(data);
    }
});
