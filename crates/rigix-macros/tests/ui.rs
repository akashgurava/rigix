#[test]
fn test_macro_ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_visibility.rs");
    t.pass("tests/ui/pass_visibility.rs");
    t.compile_fail("tests/ui/fail_getters.rs");
    t.pass("tests/ui/pass_getters.rs");
}
