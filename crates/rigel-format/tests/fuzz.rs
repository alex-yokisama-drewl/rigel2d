//! `EXEC-5` — a time-boxed fuzz run over the puppet parser.
//!
//! *No crash, no hang, bounded memory*, over inputs nobody wrote by hand. The
//! acceptance suite carries the **regression** form of this claim —
//! `execution::exec_5_a_mutated_puppet_is_refused_rather_than_crashing`, a fixed
//! set of single-byte flips — and says in terms that the campaign itself is its
//! own job. This is that job, and it is deliberately not in the acceptance
//! suite: those tests are written from the criteria without sight of an
//! implementation, and a fuzz harness is written against the thing it drives.
//!
//! # The budget, and why it is small
//!
//! **Default: half a second, or 512 inputs, whichever comes first.** That is a
//! regression gate, not a campaign — it is sized so that `cargo test` does not
//! notice it, because `docs/testing.md` is explicit that the soak is the only
//! slow job and that a suite slow enough to be switched off is the failure that
//! matters.
//!
//! A real campaign is the same code with a larger budget:
//!
//! ```text
//! RIGEL_FUZZ_SECONDS=600 RIGEL_FUZZ_INPUTS=100000 cargo test -p rigel-format --test fuzz -- --nocapture
//! ```
//!
//! | Variable | Default | What it sets |
//! |---|---|---|
//! | `RIGEL_FUZZ_SECONDS` | `0.5` | The wall-clock budget for the **loop**, checked between inputs, so a slow runner cannot fail the gate. It does not bound the time spent inside one input; [`ONE_INPUT_TIMEOUT`] does that, and a hang is a failure rather than something the budget absorbs |
//! | `RIGEL_FUZZ_INPUTS` | `512` | The number of inputs, if the budget does not run out first |
//! | `RIGEL_FUZZ_SEED` | a fixed constant | The generator's seed |
//!
//! # Reproducing a failure
//!
//! Every input is derived from the **seed and its own index alone** — the
//! generator is re-seeded per input rather than carried across the loop — so
//! input 9174 is the same bytes however many ran before it, on any machine. A
//! failure prints its seed, its index, its seed fixture and its mutation, and
//! `RIGEL_FUZZ_SEED=<seed> RIGEL_FUZZ_INPUTS=<index + 1>` reaches it again.
//!
//! The failing container is left on disk and its path is printed. A run that
//! finds nothing leaves nothing behind.

use std::fs;
use std::io::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use rigel_format::{LimitKind, declared_limit, load};

/// The acceptance suite's fixture scaffolding, reached by path rather than
/// copied: it reads the catalogue, and two readers of one catalogue that drift
/// apart is the failure `support.rs` exists to prevent.
#[path = "acceptance/support.rs"]
mod support;

#[path = "shared/counting_allocator.rs"]
mod counting;

#[global_allocator]
static ALLOCATOR: counting::Counting = counting::Counting;

/// Changed only when a run should explore different ground; a fixed seed is what
/// makes a green run mean the same thing twice.
const DEFAULT_SEED: u64 = 0x5249_4745_4c32_4446;
/// The floor an honest run must reach, whatever the clock says. The budget
/// below stops the loop only once this many inputs have been tried.
const MIN_INPUTS: usize = 64;
const DEFAULT_SECONDS: f64 = 0.5;
const DEFAULT_INPUTS: usize = 512;

/// The valid puppets every input is a mutation of.
///
/// More than one, and deliberately unalike: `rigged` carries extension data and
/// a physics subtree, `one-part` is the smallest file the format admits,
/// `rigged:physics-chain` is the deepest tree in the corpus, `rigged:ext-hostile`
/// carries path- and command-shaped strings in every string-bearing field, and
/// `rigged:future-minor` carries opaquely preserved material a mutation can
/// reach. Mutating one file only fuzzes the paths that file happens to walk.
const SEEDS: &[&str] = &[
    "rigged",
    "one-part",
    "rigged:deep",
    "rigged:physics-chain",
    "rigged:ext-hostile",
    "rigged:future-minor",
    "rigged:no-gravity",
];

// ---------------------------------------------------------------------------
// The generator
// ---------------------------------------------------------------------------

/// `splitmix64`. Seeded per input from the run's seed and the input's index, so
/// an input does not depend on the ones before it.
struct Rng(u64);

impl Rng {
    fn for_input(seed: u64, index: usize) -> Self {
        Rng(seed ^ (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value below `bound`, which must not be zero.
    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn byte(&mut self) -> u8 {
        (self.next() & 0xFF) as u8
    }

    fn pick<'a, T>(&mut self, from: &'a [T]) -> &'a T {
        &from[self.below(from.len())]
    }
}

// ---------------------------------------------------------------------------
// The mutations
// ---------------------------------------------------------------------------

/// What was done to a valid puppet to make this input.
///
/// The container half reaches the archive reader; the JSON half reaches the
/// parser, the decoder and the validator, which the container half mostly never
/// gets past — a flipped byte usually breaks a CRC and is refused there. Both
/// are needed, and a campaign that is only one of them fuzzes one module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mutation {
    /// One to eight bytes of the container flipped.
    FlipBytes,
    /// A run of the container overwritten with one repeated byte.
    Splat,
    /// The container cut short.
    Truncate,
    /// A run of bytes inserted, which shifts every offset after it.
    Insert,
    /// A run of bytes removed, likewise.
    Delete,
    /// `puppet.json` spliced as **text**, then repacked. Reaches token shapes a
    /// JSON writer cannot emit — an unterminated string, a duplicated key, a
    /// number with two exponents.
    JsonText,
    /// `puppet.json` parsed, one node of the tree rewritten, then repacked.
    /// Reaches the decoder and the validator with a structurally valid document
    /// that says something wrong.
    JsonTree,
    /// `rigel2d.json` rewritten, which is the entry read before any other.
    ManifestText,
    /// The archive rebuilt with an entry renamed, dropped or duplicated.
    Entries,
}

impl Mutation {
    const ALL: &'static [Mutation] = &[
        Mutation::FlipBytes,
        Mutation::Splat,
        Mutation::Truncate,
        Mutation::Insert,
        Mutation::Delete,
        Mutation::JsonText,
        Mutation::JsonTree,
        Mutation::ManifestText,
        Mutation::Entries,
    ];
}

/// Applies one mutation. Returns `None` when the mutation had nothing to work
/// on, which is not a failure — the next input is a different one.
fn mutate(original: &[u8], kind: Mutation, rng: &mut Rng) -> Option<Vec<u8>> {
    match kind {
        Mutation::FlipBytes => {
            let mut bytes = original.to_vec();
            for _ in 0..=rng.below(8) {
                let at = rng.below(bytes.len());
                bytes[at] ^= rng.byte() | 1;
            }
            Some(bytes)
        }
        Mutation::Splat => {
            let mut bytes = original.to_vec();
            let at = rng.below(bytes.len());
            let run = 1 + rng.below(256.min(bytes.len() - at));
            let fill = rng.byte();
            bytes[at..at + run].fill(fill);
            Some(bytes)
        }
        Mutation::Truncate => {
            let keep = rng.below(original.len());
            Some(original[..keep].to_vec())
        }
        Mutation::Insert => {
            let mut bytes = original.to_vec();
            let at = rng.below(bytes.len());
            let run: Vec<u8> = (0..1 + rng.below(64)).map(|_| rng.byte()).collect();
            bytes.splice(at..at, run);
            Some(bytes)
        }
        Mutation::Delete => {
            let mut bytes = original.to_vec();
            let at = rng.below(bytes.len());
            let run = 1 + rng.below(64.min(bytes.len() - at));
            bytes.drain(at..at + run);
            Some(bytes)
        }
        Mutation::JsonText => {
            let mut entries = unpack(original)?;
            let text = entries.iter_mut().find(|(name, _)| name == "puppet.json")?;
            text.1 = splice_text(&text.1, rng);
            Some(pack(&entries))
        }
        Mutation::JsonTree => {
            let mut entries = unpack(original)?;
            let slot = entries.iter_mut().find(|(name, _)| name == "puppet.json")?;
            let mut doc: serde_json::Value = serde_json::from_slice(&slot.1).ok()?;
            rewrite(&mut doc, rng);
            slot.1 = serde_json::to_vec(&doc).ok()?;
            Some(pack(&entries))
        }
        Mutation::ManifestText => {
            let mut entries = unpack(original)?;
            let slot = entries
                .iter_mut()
                .find(|(name, _)| name == "rigel2d.json")?;
            slot.1 = splice_text(&slot.1, rng);
            Some(pack(&entries))
        }
        Mutation::Entries => {
            let mut entries = unpack(original)?;
            match rng.below(4) {
                0 => {
                    let at = rng.below(entries.len());
                    entries.remove(at);
                }
                1 => {
                    let at = rng.below(entries.len());
                    entries[at].0 = format!("textures/{}.png", rng.next());
                }
                2 => {
                    let at = rng.below(entries.len());
                    let copy = entries[at].clone();
                    entries.push((format!("{}~", copy.0), copy.1));
                }
                _ => {
                    let at = rng.below(entries.len());
                    let name = entries[at].0.clone();
                    entries.push((name, vec![rng.byte(); 1 + rng.below(32)]));
                }
            }
            Some(pack(&entries))
        }
    }
}

/// Text-level surgery: the shapes a JSON writer cannot produce.
fn splice_text(original: &[u8], rng: &mut Rng) -> Vec<u8> {
    let mut text = original.to_vec();
    if text.is_empty() {
        return text;
    }
    match rng.below(6) {
        // A span removed, which unbalances whatever it was inside.
        0 => {
            let at = rng.below(text.len());
            let run = 1 + rng.below(64.min(text.len() - at));
            text.drain(at..at + run);
        }
        // A span repeated, which duplicates keys and commas.
        1 => {
            let at = rng.below(text.len());
            let run = 1 + rng.below(64.min(text.len() - at));
            let copy = text[at..at + run].to_vec();
            text.splice(at..at, copy);
        }
        // A token a well-formed writer never emits.
        2 => {
            let at = rng.below(text.len());
            /// Shapes a conforming JSON writer cannot emit: non-finite
            /// literals, an exponent that overflows a double, numbers in a
            /// notation JSON does not define, a lone surrogate, a NUL, and
            /// bytes that are not UTF-8 at all.
            const TOKENS: &[&[u8]] = &[
                b"NaN",
                b"Infinity",
                b"-Infinity",
                b"1e999",
                b"0x10",
                b"00",
                b"1.2.3",
                b"+1",
                b".5",
                b"'",
                b"\\u",
                b"\\uD800",
                b"\0",
                b"\xff\xfe",
                b"--",
                b"1e",
            ];
            let token = *rng.pick(TOKENS);
            text.splice(at..at, token.iter().copied());
        }
        // A bracket, which changes the shape of everything after it.
        3 => {
            let at = rng.below(text.len());
            text.insert(at, *rng.pick(b"{}[],:\"".as_slice()));
        }
        // Cut short, so every container is unterminated.
        4 => {
            let keep = rng.below(text.len());
            text.truncate(keep);
        }
        // A deeply nested value, against the parser's depth cap.
        _ => {
            let depth = 1 + rng.below(4096);
            let mut nested = Vec::with_capacity(depth * 2 + 2);
            nested.extend(std::iter::repeat_n(b'[', depth));
            nested.push(b'1');
            nested.extend(std::iter::repeat_n(b']', depth));
            let at = rng.below(text.len());
            text.splice(at..at, nested);
        }
    }
    text
}

/// Rewrites one node of a parsed document, chosen by random descent.
fn rewrite(value: &mut serde_json::Value, rng: &mut Rng) {
    use serde_json::Value;

    // Stop here, or descend into a child. Stopping at the root sometimes is
    // deliberate: a whole document replaced by a number is an input too.
    let children = match value {
        Value::Array(items) => items.len(),
        Value::Object(fields) => fields.len(),
        _ => 0,
    };
    if children > 0 && rng.below(children + 1) != 0 {
        match value {
            Value::Array(items) => {
                let at = rng.below(items.len());
                rewrite(&mut items[at], rng);
            }
            Value::Object(fields) => {
                let at = rng.below(fields.len());
                let key = fields.keys().nth(at).expect("a key").clone();
                match rng.below(8) {
                    // Remove the member, which is how a required field goes
                    // missing and an absent-when-default field is tested.
                    0 => {
                        fields.remove(&key);
                    }
                    // Rename it, which is how unmarked data the core format does
                    // not define arrives.
                    1 => {
                        if let Some(held) = fields.remove(&key) {
                            fields.insert(format!("{key}~{}", rng.below(16)), held);
                        }
                    }
                    _ => rewrite(fields.get_mut(&key).expect("the member"), rng),
                }
            }
            _ => unreachable!("children is zero for a scalar"),
        }
        return;
    }

    // An array sometimes grows or shrinks rather than having a member rewritten:
    // a collection's length is what a count limit and a binding's value count
    // are about.
    if let Value::Array(items) = value
        && !items.is_empty()
        && rng.below(3) == 0
    {
        match rng.below(3) {
            0 => {
                let at = rng.below(items.len());
                items.remove(at);
            }
            1 => {
                let at = rng.below(items.len());
                let copy = items[at].clone();
                items.push(copy);
            }
            _ => items.reverse(),
        }
        return;
    }

    *value = wild(rng);
}

/// A value chosen to be wrong in a way the loader has to have an answer for.
fn wild(rng: &mut Rng) -> serde_json::Value {
    use serde_json::Value;
    match rng.below(14) {
        0 => Value::Null,
        1 => Value::Bool(rng.next() & 1 == 0),
        2 => Value::Array(Vec::new()),
        3 => Value::Object(serde_json::Map::new()),
        4 => Value::String(String::new()),
        // An identifier that is reserved, out of range, or somebody else's.
        5 => Value::from(0u64),
        6 => Value::from(u64::MAX),
        7 => Value::from(rng.next()),
        // Numbers at the edges of what a float survives.
        8 => Value::from(-1i64),
        9 => Value::from(f64::MAX),
        10 => Value::from(f64::MIN_POSITIVE),
        11 => Value::from(-0.0f64),
        // A string where a number or an object belongs.
        12 => Value::String("../../etc/passwd".to_owned()),
        _ => {
            // Nesting, against the decoder rather than the parser.
            let mut nested = Value::from(1u64);
            for _ in 0..rng.below(64) {
                nested = Value::Array(vec![nested]);
            }
            nested
        }
    }
}

// ---------------------------------------------------------------------------
// The container
// ---------------------------------------------------------------------------

/// Every entry of an archive, as name and bytes, or `None` if it will not open.
///
/// Mutations that corrupt the container cannot be repacked, and that is fine:
/// they are the container mutations' job, not this one's.
fn unpack(bytes: &[u8]) -> Option<Vec<(String, Vec<u8>)>> {
    use std::io::Read as _;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).ok()?;
    let mut out = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).ok()?;
        let name = entry.name().ok()?.into_owned();
        let mut payload = Vec::new();
        entry.read_to_end(&mut payload).ok()?;
        out.push((name, payload));
    }
    Some(out)
}

/// Writes entries back into an archive.
///
/// The compression level is left unset on purpose: with only the
/// `deflate-flate2` feature on, `compression_level(Some(6))` fails at runtime
/// with *Unsupported compression level* — `docs/wip/s2-format.md`'s task 1
/// traps.
fn pack(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, payload) in entries {
        if writer.start_file(name, options).is_err() {
            continue;
        }
        let _ = writer.write_all(payload);
    }
    writer
        .finish()
        .map(std::io::Cursor::into_inner)
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// The run
// ---------------------------------------------------------------------------

fn env_or<T: std::str::FromStr>(name: &str, fallback: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(fallback)
}

/// What one input did.
struct Verdict {
    accepted: bool,
    panicked: bool,
    peak: u64,
}

/// How long one input may take before the run is called a hang.
///
/// Generous against the slowest hosted runner — no legitimate input comes near
/// it — and the point is only that an input which never returns is caught as
/// the defect it is, rather than running until CI's own job timeout kills the
/// whole workflow with no attribution.
const ONE_INPUT_TIMEOUT: Duration = Duration::from_secs(30);

/// Runs one input **on a worker thread, with a timeout**.
///
/// The loop's deadline is checked *between* inputs, so it bounds the number of
/// inputs and not the time spent inside one. Until this existed, the module
/// table's claim that "a hang cannot outlive it" was false in exactly the case
/// it was written for, and the comment in the loop — "every input is bounded
/// by the loader's own limits, so reaching it cannot hang CI either" — was the
/// assumption that made the claim look safe. Task 11 falsified it: a 4 KB
/// input took 23 seconds, and a larger one of the same shape would have taken
/// hours.
///
/// A thread that does not return cannot be killed, so a timeout here fails the
/// run and leaves the thread to die with the process. The counting allocator's
/// counters are per thread, which is what makes measuring on a worker sound.
fn drive(path: &Path) -> Option<Verdict> {
    let owned = path.to_path_buf();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut accepted = false;
        let mut panicked = false;
        let peak = counting::peak_bytes(|| {
            match catch_unwind(AssertUnwindSafe(|| load(&owned))) {
                Ok(outcome) => accepted = outcome.is_ok(),
                Err(_) => panicked = true,
            };
        });
        let _ = sender.send(Verdict {
            accepted,
            panicked,
            peak,
        });
    });
    receiver.recv_timeout(ONE_INPUT_TIMEOUT).ok()
}

/// `EXEC-5` — every input ends in a refusal or a successful load. Never a panic,
/// never an allocation past the loader's own declared budget, never a hang.
///
/// The three failures are asserted separately because they are different bugs: a
/// panic is a parser that trusted something, an allocation past the budget is a
/// limit that was checked after the fact rather than before, and the hang is
/// bounded by the deadline rather than asserted — a loop that does not return
/// cannot be caught by a value it does not produce.
#[test]
fn exec_5_a_fuzz_run_over_the_puppet_parser() {
    let seed: u64 = env_or("RIGEL_FUZZ_SEED", DEFAULT_SEED);
    let seconds: f64 = env_or("RIGEL_FUZZ_SECONDS", DEFAULT_SECONDS);
    let inputs: usize = env_or("RIGEL_FUZZ_INPUTS", DEFAULT_INPUTS);

    // The whole load's own declared budget. A mutated file may honestly be
    // larger than the one it came from, so the bound asserted is the loader's
    // rather than the seed fixture's size — but it is the loader's *stated*
    // bound, so an unbounded allocation fails here rather than being swallowed
    // by a figure chosen to accommodate it.
    let budget = declared_limit(LimitKind::TotalLoadMemory).expect("a declared figure");

    let originals: Vec<(&str, Vec<u8>)> = SEEDS
        .iter()
        .map(|name| (*name, support::bytes(&support::fixture(name))))
        .collect();

    let dir = support::scratch("fuzz");
    let path = dir.join("input.rigel2d");
    let deadline = Instant::now() + Duration::from_secs_f64(seconds);
    let started = Instant::now();

    let mut ran = 0usize;
    let mut accepted = 0usize;
    let mut skipped = 0usize;
    let mut by_kind = [0usize; 9];

    for index in 0..inputs {
        // The deadline binds only once the floor is met. Coupling the two the
        // other way made this gate fail on a hosted runner that is slower than
        // the machine it was written on — 49 inputs in 0.5s, tripping the floor
        // below. A gate whose verdict depends on how loaded the runner is is not
        // a gate. The floor is small, and an input that never returns is caught
        // by `ONE_INPUT_TIMEOUT` rather than by this deadline, which cannot see
        // inside a call that has not come back.
        if ran >= MIN_INPUTS && Instant::now() >= deadline {
            break;
        }
        let mut rng = Rng::for_input(seed, index);
        let (name, original) = &originals[index % originals.len()];
        let kind = *rng.pick(Mutation::ALL);
        let Some(mutated) = mutate(original, kind, &mut rng) else {
            skipped += 1;
            continue;
        };
        fs::write(&path, &mutated).expect("the input writes");

        let Some(verdict) = drive(&path) else {
            let kept = dir.join(format!("hanging-{index}.rigel2d"));
            fs::write(&kept, &mutated).expect("the hanging input is kept");
            panic!(
                "seed {seed:#x}, input {index}, {name} mutated by {kind:?}: \
                 one input did not return within {ONE_INPUT_TIMEOUT:?}, which is \
                 a hang. The input is at {kept:?}; reach it again with \
                 RIGEL_FUZZ_SEED={seed} RIGEL_FUZZ_INPUTS={}",
                index + 1
            );
        };
        ran += 1;
        by_kind[Mutation::ALL
            .iter()
            .position(|k| *k == kind)
            .expect("a kind")] += 1;
        if verdict.accepted {
            accepted += 1;
        }

        if verdict.panicked || verdict.peak > budget {
            let kept = dir.join(format!("failing-{index}.rigel2d"));
            fs::write(&kept, &mutated).expect("the failing input is kept");
            panic!(
                "seed {seed:#x}, input {index}, {name} mutated by {kind:?}: \
                 {}. The input is at {kept:?}; reach it again with \
                 RIGEL_FUZZ_SEED={seed} RIGEL_FUZZ_INPUTS={}",
                if verdict.panicked {
                    "the load panicked".to_owned()
                } else {
                    format!(
                        "the load held {} bytes, past the {budget} it declares",
                        verdict.peak
                    )
                },
                index + 1
            );
        }
    }

    let elapsed = started.elapsed();
    let coverage: Vec<String> = Mutation::ALL
        .iter()
        .zip(by_kind)
        .map(|(kind, count)| format!("{kind:?} {count}"))
        .collect();
    println!(
        "fuzz: seed {seed:#x}, {ran} inputs in {:.2}s ({accepted} loaded, \
         {skipped} unrepackable), budget {seconds}s / {inputs} inputs\n  {}",
        elapsed.as_secs_f64(),
        coverage.join(", ")
    );

    assert!(
        ran >= MIN_INPUTS,
        "only {ran} inputs ran in {seconds}s: the budget is too small to be a \
         regression gate, and a fuzz run nobody notices passing is one nobody \
         notices doing nothing"
    );
    assert!(
        accepted > 0,
        "not one of {ran} inputs loaded: every mutation is being refused at the \
         container, so nothing past the archive reader was fuzzed at all"
    );
    assert!(
        by_kind.iter().all(|count| *count > 0),
        "a mutation kind produced no input: {}",
        coverage.join(", ")
    );
    let _: PathBuf = path;
}
