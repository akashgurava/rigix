#[test]
fn test_entity_macro_ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_pub_field.rs");
    t.compile_fail("tests/ui/fail_multi_pub_fields.rs");
    t.pass("tests/ui/pass_private_fields.rs");
    t.pass("tests/ui/pass_generated_getters.rs");
}
