//! The standalone validator command `VAL-13` requires.
//!
//! *One validator gates load and save, and runs as a standalone command.* This
//! is that command. It wraps [`rigel_format::validate_file`] and adds nothing:
//! a second code path reaching its own verdict would be a second definition of a
//! valid puppet, which is the failure the criterion exists to prevent.
//!
//! # The contract the acceptance suite asserts
//!
//! ```text
//! rigel-validate <path>
//! ```
//!
//! | | |
//! |---|---|
//! | Accepted | prints `accepted` on one line to stdout, exits `0` |
//! | Rejected | prints `rejected: <rule> <detail>` to stdout, exits `1` |
//! | No path, or more than one | prints usage to stderr, exits `2` |
//!
//! The verdict goes to **stdout** and the exit status carries it too, so that
//! neither a shell pipeline nor a caller reading the stream has to parse prose.
//! The rejection line is [`rigel_format::Rejection`]'s `Display`, which is
//! bounded (`VAL-12`).
//!
//! # What it deliberately does not do
//!
//! It takes a path and prints a verdict. It does not write, repair, convert or
//! fetch anything, and it reads nothing but the file named — the whole of
//! [I1](../../../docs/invariants.md) in a program that exists to be pointed at
//! files from strangers.

use std::path::PathBuf;
use std::process::ExitCode;

/// Exit `2`, which is neither verdict: the command was not asked a question it
/// could answer.
const MISUSE: u8 = 2;

fn main() -> ExitCode {
    let arguments: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [path] = arguments.as_slice() else {
        eprintln!("usage: rigel-validate <path>");
        eprintln!("prints `accepted`, or `rejected: <rule>` and why, for one puppet file.");
        return ExitCode::from(MISUSE);
    };

    match rigel_format::load(path) {
        Ok(loaded) => {
            println!("accepted");
            report(&loaded.report);
            ExitCode::SUCCESS
        }
        Err(rejection) => {
            // `Display` already opens with `rejected: <rule>` and is bounded.
            println!("{rejection}");
            ExitCode::FAILURE
        }
    }
}

/// What the file said that a person should know, on **stderr**.
///
/// The verdict and the report are different things and go to different streams:
/// a pipeline reading stdout gets one line it never has to parse, while a person
/// running the command sees what opening this file would have told them. `MSG-2`
/// is a gate on this text — whether it misleads — and a gate needs something to
/// look at, which is why it is printed at all.
///
/// It never promises the file will be preserved intact and never implies that a
/// missing extension can be installed from here: nothing can be, and
/// [I6](../../../docs/invariants.md) is why.
fn report(report: &rigel_format::LoadReport) {
    if let Some(gap) = report.version_gap {
        eprintln!(
            "note: written by a later build — the file declares {}.{}, this build knows {}.{}",
            gap.found.major, gap.found.minor, gap.known.major, gap.known.minor
        );
    }
    for notice in &report.unrecognised_extensions {
        eprintln!(
            "note: carries data for {} ({}), version {}, which this build does not implement",
            notice.name, notice.namespace, notice.version
        );
    }
    match report.stale_bindings.len() {
        0 => {}
        1 => eprintln!(
            "note: 1 binding no longer matches the mesh it applies to, and is kept as authored"
        ),
        n => eprintln!(
            "note: {n} bindings no longer match the meshes they apply to, and are kept as authored"
        ),
    }
    match report.unreadable_textures.len() {
        0 => {}
        1 => eprintln!(
            "note: 1 texture payload could not be decoded, and is carried through unchanged"
        ),
        n => eprintln!(
            "note: {n} texture payloads could not be decoded, and are carried through unchanged"
        ),
    }
    for warning in &report.warnings {
        eprintln!("warning: {warning}");
    }
}
