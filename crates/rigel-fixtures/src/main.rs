//! Writes the hand-authored puppet fixture corpus to a directory.
//!
//!     cargo run -p rigel-fixtures -- <directory>
//!
//! The committed corpus lives in `crates/rigel-format/tests/fixtures`, and
//! `scripts/fixtures-check.sh` regenerates into a temporary directory and
//! asserts the committed artifacts are reproduced byte for byte. Nothing here
//! reads a clock, an environment variable or a file, so a rebuild on any
//! machine produces the same bytes.

mod archive;
mod fixtures;
mod image;
mod json;
mod puppet;
mod rawzip;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let Some(directory) = args.next() else {
        eprintln!("usage: rigel-fixtures <directory>");
        return ExitCode::FAILURE;
    };
    if args.next().is_some() {
        eprintln!("usage: rigel-fixtures <directory>");
        return ExitCode::FAILURE;
    }
    let directory = PathBuf::from(directory);

    if let Err(error) = std::fs::create_dir_all(&directory) {
        eprintln!("cannot create {}: {error}", directory.display());
        return ExitCode::FAILURE;
    }

    let corpus = fixtures::all();
    let mut index = Vec::new();
    for fixture in &corpus {
        let path = directory.join(&fixture.file);
        if let Err(error) = std::fs::write(&path, &fixture.bytes) {
            eprintln!("cannot write {}: {error}", path.display());
            return ExitCode::FAILURE;
        }
        index.push(serde_json::json!({
            "expect": fixture.expect.as_str(),
            "file": fixture.file,
            "name": fixture.name,
            "note": fixture.note,
        }));
    }

    // The catalogue, as data rather than prose: what each fixture is called in
    // `docs/requirements/puppet-acceptance.md`, which file carries it, what a
    // conforming loader does with it, and the one thing that makes it what it
    // is. A test reaches a fixture through this rather than guessing a path.
    let catalogue = serde_json::to_string_pretty(&serde_json::json!({
        "fixtures": index,
        "version": {"major": puppet::VERSION_MAJOR, "minor": puppet::VERSION_MINOR},
    }))
    .expect("catalogue is serialisable");
    let path = directory.join("catalogue.json");
    if let Err(error) = std::fs::write(&path, catalogue + "\n") {
        eprintln!("cannot write {}: {error}", path.display());
        return ExitCode::FAILURE;
    }

    println!(
        "{} fixtures written to {}",
        corpus.len(),
        directory.display()
    );
    ExitCode::SUCCESS
}
