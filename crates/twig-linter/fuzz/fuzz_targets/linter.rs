#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../tests/support/fixes.rs"]
mod fixes;

fuzz_target!(|data: &[u8]| {
    fixes::assert_fixes_settle(data);
});
