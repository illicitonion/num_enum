use std::error::Error;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[rustversion::all(nightly)]
const NIGHTLY: bool = true;

#[rustversion::not(nightly)]
const NIGHTLY: bool = false;

#[test]
fn trybuild() {
    let directory = PathBuf::from("tests/try_build");

    let mut _renamer = None;

    if NIGHTLY {
        // Sometimes error messages change on nightly - allow alternate errors on nightly.
        _renamer = Some(Renamer::rename(directory.join("compile_fail")).unwrap());
    }

    let fail = trybuild::TestCases::new();
    fail.compile_fail(directory.join("compile_fail/*.rs"));

    if NIGHTLY == false {
        fail.compile_fail(directory.join("compile_fail/*.rs"));
    }

    let pass = trybuild::TestCases::new();
    pass.pass(directory.join("pass/*.rs"));
}

struct Renamer(Vec<PathBuf>);

impl Renamer {
    const STDERR_EXTENSION: &'static str = "stderr";
    const NIGHTLY_EXTENSION: &'static str = "stderr_nightly";
    const NON_NIGHTLY_BACKUP_EXTENSION: &'static str = "stderr_non_nightly_backup";

    fn rename(dir: PathBuf) -> anyhow::Result<Self> {
        let nightly_paths = WalkDir::new(dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|dir_entry| {
                let dir_entry = match dir_entry {
                    Ok(dir_entry) => dir_entry,
                    Err(err) => return Some(Err(err)),
                };
                let path = dir_entry.path();
                if let Some(file_name) = path.file_name() {
                    if Path::new(file_name).extension() == Some(Renamer::NIGHTLY_EXTENSION.as_ref())
                    {
                        return Some(Ok(path.to_path_buf()));
                    }
                }
                None
            })
            .collect::<Result<Vec<_>, _>>()?;
        // Create early so that if we end up returning an error this gets dropped and undoes any
        // already-done renames.
        let renamer = Renamer(nightly_paths);

        for nightly_path in &renamer.0 {
            std::fs::rename(
                nightly_path.with_extension(Renamer::STDERR_EXTENSION),
                nightly_path.with_extension(Renamer::NON_NIGHTLY_BACKUP_EXTENSION),
            )?;
            std::fs::rename(
                nightly_path.with_extension(Renamer::NIGHTLY_EXTENSION),
                nightly_path.with_extension(Renamer::STDERR_EXTENSION),
            )?;
        }
        Ok(renamer)
    }
}

impl Drop for Renamer {
    fn drop(&mut self) {
        for path in &self.0 {
            ignore_error(std::fs::rename(
                path.with_extension(Renamer::STDERR_EXTENSION),
                path.with_extension(Renamer::NIGHTLY_EXTENSION),
            ));
            ignore_error(std::fs::rename(
                path.with_extension(Renamer::NON_NIGHTLY_BACKUP_EXTENSION),
                path.with_extension(Renamer::STDERR_EXTENSION),
            ));
        }
    }
}

fn ignore_error<T, E: Error>(result: Result<T, E>) {
    if let Err(err) = result {
        eprintln!("Ignoring error: {}", err);
    }
}
