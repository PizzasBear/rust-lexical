#![no_main]
use libfuzzer_sys::fuzz_target;
use lexical_parse_integer::FromLexical;

fuzz_target!(|data: &[u8]| {
    let _ = i16::from_lexical(data);
});
