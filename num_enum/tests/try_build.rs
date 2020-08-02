#[test]
fn failing_compiles() {
    let fail = trybuild::TestCases::new();
    fail.compile_fail("tests/try_build/compile_fail/*.rs");

    // let pass = trybuild::TestCases::new();
    // pass.pass("tests/try_build/pass/*.rs");
}
