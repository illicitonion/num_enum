use std::path::Path;

#[test]
fn stress_test() {
    let pass = trybuild::TestCases::new();
    pass.pass(Path::new("tests/stress_tests/*.rs"));
}
