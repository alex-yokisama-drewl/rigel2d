//! Strict JSON, kept as it arrived.
//!
//! # Why this crate parses JSON itself
//!
//! Two requirements a general-purpose JSON library does not meet together.
//!
//! **A number is kept as the text it arrived as.** `docs/format.md` requires
//! material this build does not understand to survive a save *byte for byte*,
//! and a number that has been through anybody's float formatter has not.
//! `1e400` inside opaquely preserved material must come back as `1e400` — not
//! as `inf`, and not as a parse failure, because nothing ever looks inside that
//! material (`EXT-9`). A value this build **does** interpret is converted to
//! `f64` at the point of interpretation, where overflowing to infinity is
//! exactly what [`crate::Rule::NumbersFinite`] is for.
//!
//! **The parse is strict.** A file is traded between strangers
//! ([I1](../../../docs/invariants.md)), so everything JSON does not define is
//! refused rather than guessed at: a bare `NaN` or `Infinity` token, a trailing
//! comma, a control character in a string, an invalid escape, invalid UTF-8, a
//! duplicate member name, and nesting past [`MAX_DEPTH`].
//!
//! Nothing here allocates ahead of what it has read, and the parser is
//! recursive-descent with an explicit depth cap, so a deeply nested file is
//! refused rather than overflowing the stack (`EXEC-5`).

use std::fmt::Write as _;

/// The deepest nesting the parser accepts.
///
/// A puppet's deepest shape is a binding's deformation values — puppet, array,
/// binding, values, keypoint, vertex, coordinate — which is seven. The cap is
/// generous against that and is what stops a hostile file recursing this parser
/// into a stack overflow.
///
/// It is **not** a [`crate::LimitKind`]: every kind there is a size or count of
/// something the format defines, and JSON nesting is a property of the encoding
/// rather than of the puppet. Reported as a gap in `docs/wip/s2-format.md`.
pub const MAX_DEPTH: usize = 64;

/// A JSON value, with numbers held as the text they arrived as.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    /// The literal text of a number, exactly as the file spelled it.
    Number(String),
    String(String),
    Array(Vec<Json>),
    /// Members in the order they arrived. Canonical ordering is applied when
    /// the value is written, never on the way in.
    Object(Vec<(String, Json)>),
    /// Text that is already canonical JSON, written through untouched.
    ///
    /// **The parser never produces one.** It exists for [`crate::save`], which
    /// has to put material this build does not understand back exactly as it
    /// arrived: an extension's `data`, a later-version file's unmarked members,
    /// a node of a kind this build cannot read. Those are held as the bytes
    /// [`Json::to_canonical_bytes`] produced at load, and re-emitting them
    /// through our own number formatter would damage values we by definition do
    /// not understand (`EXT-4`, `EXT-9`, `docs/format.md`).
    Raw(String),
}

impl Json {
    /// What this value is, for a message naming what was found instead.
    pub fn kind(&self) -> &'static str {
        match self {
            Json::Null => "null",
            Json::Bool(_) => "a boolean",
            Json::Number(_) => "a number",
            Json::String(_) => "a string",
            Json::Array(_) => "an array",
            Json::Object(_) => "an object",
            Json::Raw(_) => "preserved text",
        }
    }

    /// The member of an object, or `None` for anything else.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(members) => members
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&[(String, Json)]> {
        match self {
            Json::Object(members) => Some(members),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// The number this value spells, at the precision the runtime evaluates in.
    ///
    /// A literal too large for `f64` yields an infinity rather than an error,
    /// which is deliberate: *that* is what [`crate::Rule::NumbersFinite`]
    /// rejects, and rejecting it here instead would refuse the file as
    /// malformed and lose the rule the criteria name (`VAL-6`).
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Number(text) => text.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// The non-negative integer this value spells, or `None` if it is not one.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Json::Number(text) => text.parse::<u64>().ok(),
            _ => None,
        }
    }

    /// This value in canonical form: members sorted by name, no insignificant
    /// whitespace, numbers as the text they arrived as, UTF-8
    /// (`docs/format.md` — *Canonical serialisation*).
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = String::new();
        self.write_canonical(&mut out);
        out.into_bytes()
    }

    /// Appends this value in canonical form.
    pub fn write_canonical(&self, out: &mut String) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(true) => out.push_str("true"),
            Json::Bool(false) => out.push_str("false"),
            Json::Number(text) | Json::Raw(text) => out.push_str(text),
            Json::String(text) => write_string(text, out),
            Json::Array(items) => {
                out.push('[');
                for (n, item) in items.iter().enumerate() {
                    if n > 0 {
                        out.push(',');
                    }
                    item.write_canonical(out);
                }
                out.push(']');
            }
            Json::Object(members) => {
                let mut ordered: Vec<&(String, Json)> = members.iter().collect();
                ordered.sort_by(|a, b| a.0.cmp(&b.0));
                out.push('{');
                for (n, (name, value)) in ordered.into_iter().enumerate() {
                    if n > 0 {
                        out.push(',');
                    }
                    write_string(name, out);
                    out.push(':');
                    value.write_canonical(out);
                }
                out.push('}');
            }
        }
    }
}

/// A JSON string, escaped the way `serde_json` escapes one — the shortest form
/// that is valid JSON, with the two-character escapes preferred and everything
/// else below `0x20` as `\u00XX`.
fn write_string(text: &str, out: &mut String) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Why a document is not JSON this build will read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// Bounded, and for a person to read.
    pub detail: String,
    /// Set when the document was refused for exceeding
    /// [`LimitKind::ParseTreeBytes`] rather than for being malformed. The
    /// caller reports it as the limit it is, so that "this file is too big for
    /// this build" is never dressed up as "this file is not JSON".
    pub over_budget: bool,
}

/// Parses a whole document, which must be one value and nothing after it.
pub fn parse(bytes: &[u8]) -> Result<Json, ParseError> {
    let text = std::str::from_utf8(bytes).map_err(|e| ParseError {
        detail: format!("not UTF-8: {e}"),
        over_budget: false,
    })?;
    let budget = crate::limits::declared_limit(crate::limits::LimitKind::ParseTreeBytes).ok_or(
        ParseError {
            detail: String::from(
                "no figure is declared for the parse-tree limit, and an undeclared                  limit is a defect rather than permission",
            ),
            over_budget: true,
        },
    )?;
    let mut parser = Parser {
        text,
        bytes: text.as_bytes(),
        at: 0,
        budget,
        spent: 0,
    };
    parser.skip_whitespace();
    let value = parser.value(0)?;
    parser.skip_whitespace();
    if parser.at != parser.bytes.len() {
        return Err(parser.fail("trailing content after the document"));
    }
    Ok(value)
}

struct Parser<'a> {
    /// The document as text. Validated once, by [`parse`], so that slicing it
    /// is a boundary check rather than a second validation.
    text: &'a str,
    bytes: &'a [u8],
    at: usize,
    /// Heap bytes this tree may occupy, from
    /// [`LimitKind::ParseTreeBytes`](crate::limits::LimitKind::ParseTreeBytes).
    budget: u64,
    /// Charged as the tree is built, so the limit bites **during** the parse.
    /// Checking afterwards is checking after the allocation, which is the
    /// defect this exists to prevent.
    spent: u64,
}

impl Parser<'_> {
    fn fail(&self, what: &str) -> ParseError {
        ParseError {
            detail: format!("{what}, at byte {}", self.at),
            over_budget: false,
        }
    }

    /// Charges the tree's running cost and refuses once it passes the budget.
    ///
    /// The figures are what the values actually occupy: a [`Json`] is a 32-byte
    /// enum, and a `Vec` holding them overshoots by up to double while it
    /// grows, so an element is charged twice its own size. This is an estimate
    /// and is meant to be one — the alternative is a counting allocator, which
    /// a library cannot impose on the binaries that embed it.
    fn charge(&mut self, bytes: u64) -> Result<(), ParseError> {
        self.spent = self.spent.saturating_add(bytes);
        if self.spent > self.budget {
            return Err(ParseError {
                detail: format!(
                    "the parsed structure needs more than {} bytes, which is the \
                     most this build will hold for one entry",
                    self.budget
                ),
                over_budget: true,
            });
        }
        Ok(())
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), ParseError> {
        if self.peek() == Some(byte) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.fail(&format!("expected {:?}", byte as char)))
        }
    }

    fn literal(&mut self, word: &str) -> Result<(), ParseError> {
        if self.bytes[self.at..].starts_with(word.as_bytes()) {
            self.at += word.len();
            Ok(())
        } else {
            Err(self.fail(&format!("expected {word}")))
        }
    }

    fn value(&mut self, depth: usize) -> Result<Json, ParseError> {
        if depth > MAX_DEPTH {
            return Err(self.fail("nested deeper than this build will parse"));
        }
        // Every value costs its own enum, plus the slot it will occupy in a
        // parent that overshoots as it grows.
        self.charge(2 * std::mem::size_of::<Json>() as u64)?;
        match self.peek() {
            None => Err(self.fail("the document ends where a value was expected")),
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => Ok(Json::String(self.string()?)),
            Some(b't') => {
                self.literal("true")?;
                Ok(Json::Bool(true))
            }
            Some(b'f') => {
                self.literal("false")?;
                Ok(Json::Bool(false))
            }
            Some(b'n') => {
                self.literal("null")?;
                Ok(Json::Null)
            }
            Some(b'-' | b'0'..=b'9') => self.number(),
            // `NaN` and `Infinity` land here. They are what a writer that is
            // not ours emits for a value JSON cannot express, and refusing
            // them is the point: see `rigged:nan` in `docs/wip/s2-format.md`.
            Some(other) => Err(self.fail(&format!("{:?} begins no JSON value", char::from(other)))),
        }
    }

    fn object(&mut self, depth: usize) -> Result<Json, ParseError> {
        self.expect(b'{')?;
        let mut members: Vec<(String, Json)> = Vec::new();
        // Beside the ordered members, so that detecting a repeat does not cost
        // a scan of everything already seen. A linear scan per member is
        // quadratic in an object whose arity nothing bounds.
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.at += 1;
            return Ok(Json::Object(members));
        }
        loop {
            self.skip_whitespace();
            let name = self.string()?;
            // The name is held twice — once in `members`, once in `seen` —
            // plus the `String` header each time.
            self.charge(2 * (name.len() + std::mem::size_of::<String>()) as u64)?;
            if !seen.insert(name.clone()) {
                return Err(self.fail(&format!(
                    "the member {name:?} appears twice, so what it holds is undecidable"
                )));
            }
            self.skip_whitespace();
            self.expect(b':')?;
            self.skip_whitespace();
            let value = self.value(depth + 1)?;
            members.push((name, value));
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b'}') => {
                    self.at += 1;
                    return Ok(Json::Object(members));
                }
                _ => return Err(self.fail("expected ',' or '}'")),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Json, ParseError> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(Json::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value(depth + 1)?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err(self.fail("expected ',' or ']'")),
            }
        }
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.fail("the document ends inside a string"));
            };
            match byte {
                b'"' => {
                    self.at += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.at += 1;
                    self.escape(&mut out)?;
                }
                0x00..=0x1f => {
                    return Err(self.fail("an unescaped control character in a string"));
                }
                _ => {
                    // **A run at a time, not a character at a time.** The
                    // document is known-valid UTF-8 — `parse` validated it —
                    // so a run that stops at a quote, a backslash or a control
                    // byte stops on a character boundary, and copying it whole
                    // is one `memcpy`.
                    //
                    // This loop used to revalidate `self.bytes[self.at..]`
                    // with `str::from_utf8` for **every character**, which is
                    // linear in the rest of the document each time and so
                    // quadratic overall. A 4 KB file holding one long string
                    // took 23 seconds to refuse: `LIM-6`'s linear-work rule
                    // broken, and the no-hang claim with it.
                    let start = self.at;
                    while let Some(byte) = self.peek() {
                        if matches!(byte, b'"' | b'\\' | 0x00..=0x1f) {
                            break;
                        }
                        self.at += 1;
                    }
                    debug_assert!(start < self.at, "a run of at least one byte");
                    self.charge((self.at - start) as u64)?;
                    out.push_str(&self.text[start..self.at]);
                }
            }
        }
    }

    fn escape(&mut self, out: &mut String) -> Result<(), ParseError> {
        let Some(byte) = self.peek() else {
            return Err(self.fail("the document ends inside an escape"));
        };
        self.at += 1;
        match byte {
            b'"' => out.push('"'),
            b'\\' => out.push('\\'),
            b'/' => out.push('/'),
            b'b' => out.push('\u{08}'),
            b'f' => out.push('\u{0c}'),
            b'n' => out.push('\n'),
            b'r' => out.push('\r'),
            b't' => out.push('\t'),
            b'u' => {
                let first = self.hex4()?;
                let character = if (0xd800..0xdc00).contains(&first) {
                    // A high surrogate must be followed by its low half.
                    self.expect(b'\\')?;
                    self.expect(b'u')?;
                    let second = self.hex4()?;
                    if !(0xdc00..0xe000).contains(&second) {
                        return Err(self.fail("a high surrogate with no low surrogate after it"));
                    }
                    let combined =
                        0x1_0000 + ((first as u32 - 0xd800) << 10) + (second as u32 - 0xdc00);
                    char::from_u32(combined)
                } else {
                    char::from_u32(first as u32)
                };
                match character {
                    Some(character) => out.push(character),
                    None => return Err(self.fail("an escape naming no character")),
                }
            }
            other => {
                return Err(self.fail(&format!("\\{:?} is not an escape", char::from(other))));
            }
        }
        Ok(())
    }

    fn hex4(&mut self) -> Result<u16, ParseError> {
        let end = self.at + 4;
        if end > self.bytes.len() {
            return Err(self.fail("an escape cut short"));
        }
        let digits = &self.bytes[self.at..end];
        let mut value: u16 = 0;
        for digit in digits {
            let nibble = match digit {
                b'0'..=b'9' => digit - b'0',
                b'a'..=b'f' => digit - b'a' + 10,
                b'A'..=b'F' => digit - b'A' + 10,
                _ => return Err(self.fail("an escape with a non-hexadecimal digit")),
            };
            value = value * 16 + u16::from(nibble);
        }
        self.at = end;
        Ok(value)
    }

    /// `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`, kept as its own text.
    fn number(&mut self) -> Result<Json, ParseError> {
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        match self.peek() {
            Some(b'0') => self.at += 1,
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.at += 1;
                }
            }
            _ => return Err(self.fail("a number with no integer part")),
        }
        if self.peek() == Some(b'.') {
            self.at += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.fail("a number with no digit after its point"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.at += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.at += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.at += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.fail("a number with no digit in its exponent"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.at += 1;
            }
        }
        // A number is held as its own heap string, which is most of what
        // makes a dense numeric array cost many times its text.
        self.charge(((self.at - start) + std::mem::size_of::<String>()) as u64)?;
        let text = std::str::from_utf8(&self.bytes[start..self.at])
            .expect("a number is ASCII")
            .to_owned();
        Ok(Json::Number(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(text: &str) -> Json {
        parse(text.as_bytes()).unwrap_or_else(|e| panic!("{text} parses: {e:?}"))
    }

    fn refused(text: &str) -> String {
        parse(text.as_bytes())
            .expect_err(&format!("{text} must be refused"))
            .detail
    }

    #[test]
    fn a_number_keeps_the_text_it_arrived_as() {
        // The whole reason this parser exists: a stranger's number goes back
        // out as it came in, never through a float formatter.
        for literal in ["0.25", "-0.08", "1.0", "1e400", "3.5e38", "0", "-0"] {
            let value = parsed(literal);
            assert_eq!(value, Json::Number(literal.to_owned()));
            assert_eq!(value.to_canonical_bytes(), literal.as_bytes());
        }
    }

    /// `1e400` is a number this build must carry through untouched when it is
    /// opaque (`EXT-9`) and must reject when it interprets it (`VAL-6`). Both
    /// rest on it parsing and on `as_f64` overflowing rather than failing.
    #[test]
    fn an_overflowing_literal_parses_and_reads_as_infinite() {
        let value = parsed("1e400");
        assert_eq!(value.as_f64(), Some(f64::INFINITY));
        assert_eq!(value.to_canonical_bytes(), b"1e400");
    }

    #[test]
    fn the_tokens_json_does_not_define_are_refused() {
        for literal in [
            "NaN",
            "Infinity",
            "-Infinity",
            "nan",
            "+1",
            "01",
            "1.",
            ".5",
        ] {
            let _ = refused(literal);
        }
    }

    #[test]
    fn a_duplicate_member_is_refused() {
        let detail = refused(r#"{"a":1,"a":2}"#);
        assert!(detail.contains("twice"), "{detail}");
    }

    #[test]
    fn trailing_content_and_trailing_commas_are_refused() {
        let _ = refused("{} {}");
        let _ = refused("[1,]");
        let _ = refused(r#"{"a":1,}"#);
    }

    #[test]
    fn a_control_character_in_a_string_must_be_escaped() {
        let _ = refused("\"a\nb\"");
        assert_eq!(parsed(r#""a\nb""#), Json::String("a\nb".to_owned()));
    }

    #[test]
    fn invalid_utf8_is_refused() {
        let detail = parse(&[b'"', 0xff, b'"']).expect_err("refused").detail;
        assert!(detail.contains("UTF-8"), "{detail}");
    }

    /// A hostile file must not recurse the parser into a stack overflow.
    #[test]
    fn nesting_past_the_cap_is_refused_rather_than_overflowing() {
        let deep = "[".repeat(MAX_DEPTH + 4) + &"]".repeat(MAX_DEPTH + 4);
        let detail = refused(&deep);
        assert!(detail.contains("nested deeper"), "{detail}");

        let within = "[".repeat(MAX_DEPTH - 1) + &"]".repeat(MAX_DEPTH - 1);
        let _ = parsed(&within);
    }

    #[test]
    fn surrogate_pairs_and_escapes_round_trip() {
        assert_eq!(parsed(r#""😀""#), Json::String("😀".to_owned()));
        let _ = refused(r#""\ud83d""#);
        let _ = refused(r#""\uZZZZ""#);
        let _ = refused(r#""\q""#);
    }

    /// Canonical form sorts members, which is what makes *position canonical*
    /// hold for material whose *contents* are verbatim (`EXT-4`).
    #[test]
    fn canonical_form_sorts_members_and_keeps_everything_else() {
        let value = parsed(r#"{"b": 2, "a": {"d": [1, 2], "c": "x"}}"#);
        assert_eq!(
            value.to_canonical_bytes(),
            br#"{"a":{"c":"x","d":[1,2]},"b":2}"#
        );
    }

    #[test]
    fn a_string_is_escaped_the_way_the_corpus_escapes_one() {
        let value = Json::String("quote \" slash \\ tab \t bell \u{07}".to_owned());
        assert_eq!(
            String::from_utf8(value.to_canonical_bytes()).expect("UTF-8"),
            r#""quote \" slash \\ tab \t bell \u0007""#
        );
    }

    #[test]
    fn lookups_answer_only_for_the_shape_that_has_them() {
        let value = parsed(r#"{"n": 1, "s": "t", "b": true, "a": [], "o": {}}"#);
        assert_eq!(value.get("n").and_then(Json::as_u64), Some(1));
        assert_eq!(value.get("s").and_then(Json::as_str), Some("t"));
        assert_eq!(value.get("b").and_then(Json::as_bool), Some(true));
        assert_eq!(
            value.get("a").and_then(Json::as_array).map(<[_]>::len),
            Some(0)
        );
        assert!(value.get("o").and_then(Json::as_object).is_some());
        assert!(value.get("missing").is_none());
        assert!(value.get("s").and_then(Json::as_f64).is_none());
        assert_eq!(value.get("a").expect("a").kind(), "an array");
    }
}
