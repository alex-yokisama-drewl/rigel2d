//! The format version, which `rigel2d.json` carries on its own.

/// The two numbers `rigel2d.json` holds.
///
/// `docs/format.md`, *The version is readable on its own*: a major that differs
/// is refused, a higher minor opens with a warning and stays editable, a lower or
/// equal minor opens normally.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormatVersion {
    /// A difference here is a declared break in compatibility (`VER-2`).
    pub major: u32,
    /// A higher minor than [`FormatVersion::CURRENT`] licenses unmarked data,
    /// unknown node kinds and unfamiliar enumerated values (`EXT-7`, `EXT-8`,
    /// `VAL-20`).
    pub minor: u32,
}

impl FormatVersion {
    /// The version this build writes, and the one every fixture declares unless
    /// its name says otherwise.
    ///
    /// `docs/format.md` states it, and
    /// `crates/rigel-format/tests/fixtures/catalogue.json` repeats it; the
    /// acceptance suite asserts the two agree, because a corpus authored against
    /// one version and a build writing another would make every version criterion
    /// silently vacuous.
    pub const CURRENT: FormatVersion = FormatVersion { major: 1, minor: 1 };
}
