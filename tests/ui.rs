//! Tests for emitted diagnostics

#[cfg(all(test, not(miri)))]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
}
