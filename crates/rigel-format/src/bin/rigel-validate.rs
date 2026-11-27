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
//! Nothing is implemented yet — see the crate documentation.

fn main() {
    unimplemented!()
}
