use cargo_toml::Manifest;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[test]
fn msrv() {
    let workspace_root = get_workspace_root();

    let crates_to_msrvs: BTreeMap<String, String> = vec!["num_enum", "num_enum_derive"]
        .into_iter()
        .map(|crate_name| {
            let manifest_path = workspace_root.join(crate_name).join("Cargo.toml");
            (
                crate_name.to_owned(),
                Manifest::from_path(&manifest_path)
                    .map_err(|err| {
                        format!(
                            "Failed to read manifest from path {:?}: {:?}",
                            manifest_path, err
                        )
                    })
                    .unwrap()
                    .package
                    .ok_or_else(|| format!("Missing package for crate {:?}", crate_name))
                    .unwrap()
                    .rust_version()
                    .ok_or_else(|| format!("Missing rust-version for crate {:?}", crate_name))
                    .unwrap()
                    .to_owned(),
            )
        })
        .collect();

    let crates_msrvs: BTreeSet<_> = crates_to_msrvs.values().cloned().collect();
    if crates_msrvs.len() != 1 {
        panic!(
            "Want exactly one MSRV across crates, but found: {:?}",
            crates_to_msrvs
        );
    }

    let action_bytes = std::fs::read(
        workspace_root
            .join(".github")
            .join("workflows")
            .join("msrv-build.yml"),
    )
    .expect("Failed to read github workflows yaml");
    let action_file: github_action::File = serde_yaml::from_slice(&action_bytes).unwrap();

    let toolchains: BTreeSet<String> = action_file
        .jobs
        .values()
        .flat_map(|v| v.steps.iter())
        .filter_map(|step| step.with.as_ref())
        .map(|step_with| step_with.toolchain.clone())
        .collect();

    assert_eq!(toolchains, crates_msrvs);
}

#[test]
fn dep_version() {
    let workspace_root = get_workspace_root();
    let num_enum_manifest =
        Manifest::from_path(workspace_root.join("num_enum").join("Cargo.toml")).unwrap();
    let num_enum_derive_manifest =
        Manifest::from_path(workspace_root.join("num_enum_derive").join("Cargo.toml")).unwrap();

    let requested_dependency_version = num_enum_manifest
        .dependencies
        .get("num_enum_derive")
        .unwrap()
        .detail()
        .unwrap()
        .version
        .clone()
        .unwrap();
    let num_enum_derive_version = num_enum_derive_manifest.package.unwrap().version.unwrap();

    let expected_dependency_version = format!("={}", num_enum_derive_version);

    assert_eq!(expected_dependency_version, requested_dependency_version);
}

fn get_workspace_root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .to_path_buf()
}

mod github_action {
    use serde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Deserialize)]
    pub struct File {
        pub jobs: BTreeMap<String, Job>,
    }

    #[derive(Deserialize)]
    pub struct Job {
        pub steps: Vec<Step>,
    }

    #[derive(Deserialize)]
    pub struct Step {
        pub uses: Option<String>,
        pub with: Option<With>,
    }

    #[derive(Deserialize)]
    pub struct With {
        pub toolchain: String,
    }
}
