use trybuild::TestCases;

#[cfg_attr(miri, ignore)]
#[test]
fn expect_compile_error() {
    TestCases::new().compile_fail("tests/expect-compile-error/*.rs");
}
