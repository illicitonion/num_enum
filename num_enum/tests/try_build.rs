use std::path::PathBuf;

#[rustversion::all(nightly)]
const NIGHTLY: bool = true;

#[rustversion::not(nightly)]
const NIGHTLY: bool = false;

#[test]
fn trybuild() {
    let directory = PathBuf::from("tests/try_build");

    let fail = trybuild::TestCases::new();
    fail.compile_fail(directory.join("compile_fail/*.rs"));

    if NIGHTLY == false {
        fail.compile_fail(directory.join("compile_fail/*.rs"));
    }

    let pass = trybuild::TestCases::new();
    pass.pass(directory.join("pass/*.rs"));
}
