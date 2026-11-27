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

    match rigel_format::validate_file(path) {
        Ok(()) => {
            println!("accepted");
            ExitCode::SUCCESS
        }
        Err(rejection) => {
            // `Display` already opens with `rejected: <rule>` and is bounded.
            println!("{rejection}");
            ExitCode::FAILURE
        }
    }
}
