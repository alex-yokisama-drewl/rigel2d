//! Load limits — the maxima the loader enforces on a file **before allocating**.
//!
//! A load limit is not a ceiling and not an authoring limit: it is a defence
//! against a corrupt or hostile file, generous against the `stress` workload, and
//! never a choice presented to the person rigging (`LIM-5`, `docs/vocabulary.md`).
//!
//! **They are declared here and nowhere else** (`LIM-1`), and every stored
//! collection and string the loader trusts has one (`LIM-8`).
//!
//! # Where the figures come from
//!
//! `LIM-4` fixes every limit at **four times the corresponding `stress`
//! dimension**, and those dimensions are committed in
//! `docs/requirements/platform.md`, *The reference puppet and the stress
//! puppet*. S3 generates the *puppet*; the table is already there, so most of
//! these figures are derived rather than chosen. [`Figure::basis`] says which of
//! the three each one is and [`Figure::note`] says how it was reached, so that
//! nothing here is a number without a reason behind it.
//!
//! Two traps in that table, both stated in it:
//!
//! - its *Nodes* row counts tree entries **except physics**, so a node limit is
//!   four times the **sum** of the Nodes and Physics nodes rows;
//! - its *Bindings* row is marked **understated** — one Cubism keyform set
//!   carries three inochi2d bindings — so [`LimitKind::BindingCount`] is the
//!   derived figure most likely to need raising when a real rig meets it.
//!
//! # What is provisional, and what that means
//!
//! A [`Basis::Provisional`] figure is one the table has **no row for**. It is a
//! placeholder with a stated reason, not a measurement, and it is marked here
//! and nowhere else, so that landing the real figure is an edit to this table
//! rather than a hunt through the loader.
//!
//! `docs/wip/s2-format.md` named six such kinds. Walking [`LimitKind::ALL`]
//! against the table turns up more — a keypoint count, a response curve, a
//! chain's joints, an entry name, a texture dimension and extension data have no
//! dimension either — and that gap is reported there rather than papered over
//! by deriving a figure from a row that is about something else.

/// One size or count the loader refuses to trust.
///
/// A collection or a string added to the format is added here, which is what
/// bounds a kind added later on arrival rather than after someone notices
/// (`LIM-8`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LimitKind {
    /// Bytes actually decompressed out of **one archive entry**. Enforced
    /// against bytes produced, never against the entry's declared size, which is
    /// attacker-controlled (`LIM-3`).
    DecompressedPayloadBytes,
    /// Resident bytes across the whole load, successful or failed.
    TotalLoadMemory,
    /// Heap bytes the parsed JSON tree of **one entry** is allowed to occupy.
    ///
    /// Distinct from [`LimitKind::DecompressedPayloadBytes`], which bounds the
    /// *text*. A tree costs several times its text — every value is a 32-byte
    /// enum and every number is a separately allocated string — so a payload
    /// inside its own limit can still exceed the whole load's budget. Nothing
    /// downstream can refuse it, because every collection limit in `decode`
    /// runs after the tree exists.
    ParseTreeBytes,
    /// Entries in the archive.
    ArchiveEntries,
    /// Bytes in one archive entry's name.
    EntryNameBytes,
    /// Either dimension of one texture's authored or stored image.
    TextureDimension,
    /// Texture entities in one puppet.
    TextureCount,
    /// Vertices in one mesh.
    VertexCount,
    /// Triangles in one mesh.
    TriangleCount,
    /// Nodes in one puppet, physics nodes included — they are tree entries
    /// additional to the drawn ones (`LIM-4`).
    NodeCount,
    /// Depth of the node tree, counted through a physics chain like any other
    /// subtree.
    TreeDepth,
    /// Parameters in one puppet.
    ParameterCount,
    /// Axes on one parameter, and so also the length of a physics node's
    /// `output_source` and `output_scale`, which `docs/format.md` states are
    /// *per axis of that parameter*.
    AxisCount,
    /// Keypoints on one parameter.
    KeypointCount,
    /// Bindings in one puppet.
    BindingCount,
    /// Animations in one puppet.
    AnimationCount,
    /// Tracks in one animation.
    TracksPerAnimation,
    /// Keyframes in one track.
    KeyframesPerTrack,
    /// Channel mappings in one puppet.
    ChannelMappingCount,
    /// Points in one response curve.
    ResponseCurvePoints,
    /// Joints in one chain deformer.
    ChainJointCount,
    /// Bytes in any stored string.
    StringBytes,
    /// Bytes of extension data carried on one object.
    ExtensionDataBytes,
}

impl LimitKind {
    /// Every limit the loader enforces. `LIM-1`'s *declared in one place* is this
    /// slice; its *and every declared limit has a fixture just over it* half is
    /// blocked on S3.
    pub const ALL: &'static [LimitKind] = &[
        LimitKind::DecompressedPayloadBytes,
        LimitKind::TotalLoadMemory,
        LimitKind::ParseTreeBytes,
        LimitKind::ArchiveEntries,
        LimitKind::EntryNameBytes,
        LimitKind::TextureDimension,
        LimitKind::TextureCount,
        LimitKind::VertexCount,
        LimitKind::TriangleCount,
        LimitKind::NodeCount,
        LimitKind::TreeDepth,
        LimitKind::ParameterCount,
        LimitKind::AxisCount,
        LimitKind::KeypointCount,
        LimitKind::BindingCount,
        LimitKind::AnimationCount,
        LimitKind::TracksPerAnimation,
        LimitKind::KeyframesPerTrack,
        LimitKind::ChannelMappingCount,
        LimitKind::ResponseCurvePoints,
        LimitKind::ChainJointCount,
        LimitKind::StringBytes,
        LimitKind::ExtensionDataBytes,
    ];

    /// The name a rejection uses for this kind, in `kebab-case`.
    pub fn as_str(self) -> &'static str {
        match self {
            LimitKind::DecompressedPayloadBytes => "decompressed-payload-bytes",
            LimitKind::TotalLoadMemory => "total-load-memory",
            LimitKind::ParseTreeBytes => "parse-tree-bytes",
            LimitKind::ArchiveEntries => "archive-entries",
            LimitKind::EntryNameBytes => "entry-name-bytes",
            LimitKind::TextureDimension => "texture-dimension",
            LimitKind::TextureCount => "texture-count",
            LimitKind::VertexCount => "vertex-count",
            LimitKind::TriangleCount => "triangle-count",
            LimitKind::NodeCount => "node-count",
            LimitKind::TreeDepth => "tree-depth",
            LimitKind::ParameterCount => "parameter-count",
            LimitKind::AxisCount => "axis-count",
            LimitKind::KeypointCount => "keypoint-count",
            LimitKind::BindingCount => "binding-count",
            LimitKind::AnimationCount => "animation-count",
            LimitKind::TracksPerAnimation => "tracks-per-animation",
            LimitKind::KeyframesPerTrack => "keyframes-per-track",
            LimitKind::ChannelMappingCount => "channel-mapping-count",
            LimitKind::ResponseCurvePoints => "response-curve-points",
            LimitKind::ChainJointCount => "chain-joint-count",
            LimitKind::StringBytes => "string-bytes",
            LimitKind::ExtensionDataBytes => "extension-data-bytes",
        }
    }
}

/// How a figure was arrived at.
///
/// Carried per row so that the provisional ones are visible here and nowhere
/// else, and so that landing a measurement is an edit to [`FIGURES`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Basis {
    /// Four times a committed `stress` dimension, which is what `LIM-4` fixes
    /// every limit at.
    Stress,
    /// Derived from another limit or from the format's own shape, because the
    /// format bounds this kind by something already bounded here.
    Derived,
    /// **No `stress` dimension exists for this kind.** A placeholder with a
    /// stated reason, to be replaced by a measurement.
    Provisional,
    /// **Set from memory safety, not from what an author would write.**
    ///
    /// The distinction `docs/requirements/puppet.md` draws between a *limit*
    /// and a *budget*: a budget describes the workload the frame targets are
    /// measured against, a limit only stops a file from exhausting memory. A
    /// figure on this basis is deliberately far above any authored puppet, and
    /// raising it is a memory argument rather than a workload one.
    ///
    /// Not `Provisional`: nothing is missing and no measurement would replace
    /// it. Not `Stress`: a multiple of the workload would be the wrong
    /// quantity, and is how this kind came to carry a figure a one-minute
    /// animation already exceeded.
    Memory,
}

/// One kind's stated figure, or its absence.
#[derive(Clone, Copy, Debug)]
pub struct Figure {
    /// The kind this row declares.
    pub kind: LimitKind,
    /// The maximum enforced. `None` is a defect rather than *no limit* — see
    /// [`declared_limit`].
    pub limit: Option<u64>,
    /// Whether the figure is derived or provisional.
    pub basis: Basis,
    /// How it was reached, in one line, so the next reader can re-derive it.
    pub note: &'static str,
}

/// Four times a `stress` dimension, which is the multiple `LIM-4` fixes.
const fn stress(dimension: u64) -> Option<u64> {
    Some(4 * dimension)
}

/// The one place every load limit is declared (`LIM-1`).
///
/// One row per [`LimitKind`], each saying what it enforces and how that figure
/// was reached. The `stress` dimensions are
/// `docs/requirements/platform.md`'s, *The reference puppet and the stress
/// puppet*.
pub const FIGURES: &[Figure] = &[
    Figure {
        kind: LimitKind::DecompressedPayloadBytes,
        limit: Some(64 * 1024 * 1024),
        basis: Basis::Provisional,
        note: "no dimension: the table's artwork rows are totals over a puppet, not \
               the size of one entry, and it calls them its least trustworthy. 64 MiB \
               is far above any single stress-scale PNG and far below the quarter of \
               a gigabyte rigged:decompression-bomb produces",
    },
    Figure {
        kind: LimitKind::TotalLoadMemory,
        limit: Some(1024 * 1024 * 1024),
        basis: Basis::Provisional,
        note: "no dimension: half the 2 GB resident budget platform.md gives the \
               editor with a reference puppet and its source artwork open, so a file \
               needing more than this could not be held open anyway",
    },
    Figure {
        kind: LimitKind::ParseTreeBytes,
        limit: Some(256 * 1024 * 1024),
        basis: Basis::Provisional,
        note: "no dimension: a quarter of TotalLoadMemory, which leaves room for the                payload text and the decoded puppet beside the tree. Far above any                stress-scale puppet.json, whose tree is tens of megabytes, and far                below the 1.9 GiB a 63 KB archive of tiny array elements reached                before this limit existed",
    },
    Figure {
        kind: LimitKind::ArchiveEntries,
        limit: Some(4 * 300 + 2),
        basis: Basis::Derived,
        note: "the format defines one entry per texture plus rigel2d.json and \
               puppet.json, so this is the texture-count limit and two",
    },
    Figure {
        kind: LimitKind::EntryNameBytes,
        limit: Some(255),
        basis: Basis::Provisional,
        note: "no dimension: the longest name the format defines is \
               textures/<20 digits>.png, which is 33 bytes. 255 is the conventional \
               path-component maximum and is generous against that",
    },
    Figure {
        kind: LimitKind::TextureDimension,
        limit: Some(16384),
        basis: Basis::Provisional,
        note: "no dimension: the table gives source artwork as megapixels over a \
               whole puppet and calls those rows its least trustworthy. 16384 is the \
               largest dimension mainstream hardware samples, so a texture past it \
               could not be drawn whatever the limit said",
    },
    Figure {
        kind: LimitKind::TextureCount,
        limit: stress(300),
        basis: Basis::Stress,
        note: "the Parts row: a part references one texture, so distinct textures \
               cannot outnumber parts in any puppet a rigger authored",
    },
    Figure {
        kind: LimitKind::VertexCount,
        limit: stress(9_600),
        basis: Basis::Stress,
        note: "the Vertices row, which counts a whole puppet — one mesh cannot \
               exceed it",
    },
    Figure {
        kind: LimitKind::TriangleCount,
        limit: stress(14_100),
        basis: Basis::Stress,
        note: "the Triangles row, which counts a whole puppet — one mesh cannot \
               exceed it",
    },
    Figure {
        kind: LimitKind::NodeCount,
        limit: stress(500 + 40),
        basis: Basis::Stress,
        note: "the Nodes row plus the Physics nodes row. The first excludes \
               physics and the table says so: a limit derived from it alone is too \
               small by every physics node in the puppet",
    },
    Figure {
        kind: LimitKind::TreeDepth,
        limit: Some(256),
        basis: Basis::Provisional,
        note: "no dimension: the table counts tree entries, never how deep they \
               nest. 256 is far past any authored rig and far below what would \
               cost a recursive walk its stack",
    },
    Figure {
        kind: LimitKind::ParameterCount,
        limit: stress(150),
        basis: Basis::Stress,
        note: "the Parameters row",
    },
    Figure {
        kind: LimitKind::AxisCount,
        limit: Some(2),
        basis: Basis::Derived,
        note: "docs/vocabulary.md defines a parameter as a one- or two-dimensional                axis, which governs. Not a provisional figure to be measured later:                a third axis is not a large parameter, it is not a parameter",
    },
    Figure {
        kind: LimitKind::KeypointCount,
        limit: Some(256),
        basis: Basis::Provisional,
        note: "no dimension: the table counts parameters, never keypoints along \
               one. 256 positions on one axis is far past any authored rig",
    },
    Figure {
        kind: LimitKind::BindingCount,
        limit: stress(780),
        basis: Basis::Stress,
        note: "the Bindings row, which the table marks UNDERSTATED — the corpus \
               undercounts the Cubism half, so this is the derived figure most \
               likely to need raising",
    },
    Figure {
        kind: LimitKind::AnimationCount,
        limit: stress(32),
        basis: Basis::Stress,
        note: "the Animations row",
    },
    Figure {
        kind: LimitKind::TracksPerAnimation,
        limit: stress(150),
        basis: Basis::Provisional,
        note: "no dimension: the corpus counted .motion3.json files rather than the \
               curves inside them, and docs/wip/open-questions.md carries it. Set to \
               the parameter limit, which bounds it anyway — an animation carries at \
               most one track per parameter (VAL-27)",
    },
    Figure {
        kind: LimitKind::KeyframesPerTrack,
        limit: Some(65536),
        basis: Basis::Memory,
        note: "set from memory rather than from authoring, which is why it is far \
               above the measured corpus. One keyframe per frame is ordinary for \
               script-generated motion and reaches 65536 at about eighteen minutes \
               at 60fps. The earlier figure of 4096 was justified as `far past any \
               hand-made animation` and was wrong: a one-minute animation at 60fps \
               passes it at 68 seconds. docs/format.md, *Load limits*",
    },
    Figure {
        kind: LimitKind::ChannelMappingCount,
        limit: stress(150),
        basis: Basis::Derived,
        note: "the parameter limit: a parameter carries at most one base driver, so \
               mappings cannot outnumber parameters",
    },
    Figure {
        kind: LimitKind::ResponseCurvePoints,
        limit: Some(256),
        basis: Basis::Provisional,
        note: "no dimension: the table has no response-curve row. 256 points of \
               authored shaping on one channel is far past any rig",
    },
    Figure {
        kind: LimitKind::ChainJointCount,
        limit: Some(256),
        basis: Basis::Provisional,
        note: "no dimension: the table has no chain-deformer row. 256 joints along \
               one strand is far past any rig",
    },
    Figure {
        kind: LimitKind::StringBytes,
        limit: Some(4096),
        basis: Basis::Provisional,
        note: "no dimension: the table counts entities, never the length of a name. \
               The longest string the format defines is a content hash; 4096 bytes \
               is generous against every name a person types",
    },
    Figure {
        kind: LimitKind::ExtensionDataBytes,
        limit: Some(1024 * 1024),
        basis: Basis::Provisional,
        note: "no dimension: extension data is by definition not something this \
               table measured. 1 MiB on one object is generous for structured \
               values and well short of a payload smuggled through the marker",
    },
];

/// The figure enforced for this kind.
///
/// `None` means no figure has been stated, which is a defect rather than *no
/// limit*: a size the loader trusts with nothing bounding it is the evasion load
/// limits exist to close (`LIM-1`, `LIM-8`). The loader therefore **refuses** a
/// file rather than treating an absent figure as unlimited.
pub fn declared_limit(kind: LimitKind) -> Option<u64> {
    FIGURES
        .iter()
        .find(|figure| figure.kind == kind)
        .and_then(|figure| figure.limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every kind the loader trusts has a row, so a kind added to the format
    /// arrives needing a figure rather than silently having none.
    #[test]
    fn every_kind_has_a_row() {
        for kind in LimitKind::ALL {
            let rows = FIGURES.iter().filter(|f| f.kind == *kind).count();
            assert_eq!(rows, 1, "{kind:?} has {rows} rows in FIGURES");
        }
        assert_eq!(
            FIGURES.len(),
            LimitKind::ALL.len(),
            "FIGURES carries a row for every kind and no others"
        );
    }

    /// A figure, once stated, must be usable: zero is not a limit, it is a
    /// refusal of every file.
    #[test]
    fn a_stated_figure_is_usable() {
        for figure in FIGURES {
            if let Some(limit) = figure.limit {
                assert!(limit > 0, "{:?}'s limit must be usable", figure.kind);
            }
        }
    }

    #[test]
    fn the_lookup_goes_through_the_table() {
        for figure in FIGURES {
            assert_eq!(declared_limit(figure.kind), figure.limit);
        }
    }

    /// `LIM-1` in its assertable half: every kind resolves to a figure. A
    /// kind added with no figure fails here before it fails in the acceptance
    /// suite.
    #[test]
    fn every_kind_resolves_to_a_figure() {
        for kind in LimitKind::ALL {
            assert!(
                declared_limit(*kind).is_some(),
                "{kind:?} is trusted by the loader and has no figure"
            );
        }
    }

    /// Every row says how it was reached, and a provisional one says so in the
    /// one place `docs/wip/s2-format.md` asks for it to be marked.
    #[test]
    fn every_row_states_its_basis() {
        for figure in FIGURES {
            assert!(!figure.note.is_empty(), "{:?} has no note", figure.kind);
            if figure.basis == Basis::Provisional {
                assert!(
                    figure.note.contains("no dimension"),
                    "{:?} is provisional and must say which dimension is missing",
                    figure.kind
                );
            }
            if figure.basis == Basis::Memory {
                assert!(
                    figure.note.contains("memory"),
                    "{:?} is set from memory and must say so, because the figure \
                     looks arbitrary beside a workload-derived one",
                    figure.kind
                );
            }
        }
    }

    /// `LIM-4` fixes a derived limit at four times a `stress` dimension, and
    /// the two traps in that table are the ones a reader of it misses. Asserted
    /// against the figures written out here, so that editing one without the
    /// other fails.
    #[test]
    fn the_stress_derivations_are_four_times_the_committed_dimensions() {
        assert_eq!(
            declared_limit(LimitKind::NodeCount),
            Some(4 * (500 + 40)),
            "the Nodes row excludes physics, so a node limit uses the sum"
        );
        assert_eq!(declared_limit(LimitKind::ParameterCount), Some(4 * 150));
        assert_eq!(declared_limit(LimitKind::BindingCount), Some(4 * 780));
        assert_eq!(declared_limit(LimitKind::AnimationCount), Some(4 * 32));
        assert_eq!(declared_limit(LimitKind::VertexCount), Some(4 * 9_600));
        assert_eq!(declared_limit(LimitKind::TriangleCount), Some(4 * 14_100));
        assert_eq!(declared_limit(LimitKind::TextureCount), Some(4 * 300));
    }

    /// The decompression limit has to sit below what the catalogued bomb
    /// produces, or `rigged:decompression-bomb` expands to a quarter of a
    /// gigabyte before anything refuses it.
    #[test]
    fn the_payload_limit_refuses_the_catalogued_bomb() {
        let limit = declared_limit(LimitKind::DecompressedPayloadBytes).expect("a figure");
        assert!(
            limit < 256 * 1024 * 1024,
            "the bomb expands to 256 MiB and must exceed this limit"
        );
        assert!(
            declared_limit(LimitKind::TotalLoadMemory).expect("a figure") >= limit,
            "the whole load may hold at least one entry's worth"
        );
    }

    /// An entry name is bounded well below what a zip name field can express,
    /// which is what `rigged:entry-name-overlong` is built against.
    #[test]
    fn the_entry_name_limit_refuses_the_longest_name_a_zip_can_hold() {
        let limit = declared_limit(LimitKind::EntryNameBytes).expect("a figure");
        assert!(limit < u64::from(u16::MAX));
        assert!(
            limit > "textures/18446744073709551615.png".len() as u64,
            "and admits the longest name the format defines"
        );
    }

    #[test]
    fn every_kind_names_itself() {
        let mut seen: Vec<&str> = LimitKind::ALL.iter().map(|k| k.as_str()).collect();
        seen.sort_unstable();
        let before = seen.len();
        seen.dedup();
        assert_eq!(before, seen.len(), "two kinds share a name");
        assert!(seen.iter().all(|name| !name.is_empty()));
    }
}
