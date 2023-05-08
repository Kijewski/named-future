use trybuild::TestCases;
use version_check::Channel;

#[cfg_attr(miri, ignore)]
#[test]
fn expect_compile_error() {
    // RUSTFLAGS="--cfg IGNORE_CHANNEL" cargo +nightly test
    if cfg!(IGNORE_CHANNEL) || Channel::read().unwrap().is_stable() {
        TestCases::new().compile_fail("tests/expect-compile-error/*.rs");
    }
}
