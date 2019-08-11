#[test]
fn main() {
    assert!(::std::process::Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../renamed_num_enum/Cargo.toml",
        ))
        .status()
        .unwrap()
        .success())
}
