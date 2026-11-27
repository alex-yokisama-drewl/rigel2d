//! The in-memory puppet: what a load produces and what a save consumes.
//!
//! The shape mirrors `docs/format.md`, because the whole case for the format is
//! that a third party reads the JSON and understands it. It is **not** the shape
//! the frame loop evaluates — resolving identifiers to handles is
//! `rigel-runtime`'s, per [I3](../../../docs/invariants.md).

use crate::containment::NodeKind;

/// The stable value every stored reference names an entity by.
///
/// An unsigned 64-bit integer, unique within one puppet, never reused after a
/// deletion ([I3](../../../docs/invariants.md), `docs/format.md` — *Identifiers
/// are integers, not UUIDs*).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(pub u64);

impl Id {
    /// The reserved value, meaning *no reference*.
    ///
    /// An entity carrying it is invalid ([`crate::Rule::IdentifierZeroReserved`]);
    /// an optional reference holding it is simply empty, which is how a physics
    /// node drives nothing (`PHYS-4`).
    pub const NONE: Id = Id(0);
}

/// A complete rig: node tree, textures, parameters, bindings, animations.
#[derive(Clone, Debug, PartialEq)]
pub struct Puppet {
    /// The puppet's own identifier. It identifies the puppet, not the file: a
    /// copy of the file carries the same one and a *save as* issues a new one
    /// (`ID-6`, `ID-9`).
    pub id: Id,
    /// For people, and changed by a rename without any identifier moving
    /// (`ID-2`).
    pub name: String,
    /// The high-water mark the file records, which never decreases. A writer
    /// takes it from the file it read, never from a count of what it found
    /// (`docs/format.md`). An identifier above it is invalid.
    pub high_water_mark: u64,
    /// The one node with no parent. Must name a kind that contains
    /// ([`crate::may_be_root`]).
    pub root: Id,
    /// The puppet's single gravity, a magnitude along puppet-down that every
    /// physics node's own multiplier scales (`PHYS-6`).
    ///
    /// Defaulted to `1.0` when the file carries none, which is valid rather than
    /// malformed (`PHYS-10`), and written back only when it is not the default.
    pub gravity: f64,
    /// Always present, always written, even holding the only value the format
    /// defines (`DET-6`).
    pub composition_mode: CompositionMode,
    /// The flat node array, in depth-first preorder, siblings in draw order.
    ///
    /// A node of a kind this build does not know is **not** here: it is material
    /// the loader preserves opaquely and never interprets, so it is no reported
    /// entity (`EXT-7`).
    pub nodes: Vec<Node>,
    /// Ordered by identifier.
    pub parameters: Vec<Parameter>,
    /// Ordered by identifier.
    pub bindings: Vec<Binding>,
    /// Ordered by identifier.
    pub animations: Vec<Animation>,
    /// Ordered by identifier. Puppet data, edited in Live.
    pub channel_mappings: Vec<ChannelMapping>,
    /// Ordered by identifier. A texture is an entity in its own right; a part
    /// references one and several parts may reference the same.
    pub textures: Vec<TextureRecord>,
    /// Marked, namespaced values the core format does not define, carried on the
    /// puppet itself.
    pub extensions: Extensions,
}

impl Puppet {
    /// The identifier of the puppet and of every entity it contains.
    ///
    /// Enumerates **every piece** *What a puppet is* lists — nodes, textures,
    /// parameters and their keypoints, bindings, animations and their tracks,
    /// channel mappings, chain deformers and their joints — so that a kind added
    /// to that list is covered here without an edit to any test (`ID-1`, `ID-8`).
    ///
    /// Order is unspecified; callers compare as sets.
    pub fn identifiers(&self) -> Vec<Id> {
        unimplemented!()
    }

    /// Issues an identifier that has never been issued for this puppet.
    ///
    /// Takes the next value from [`Puppet::high_water_mark`] and advances it, so
    /// that no identifier is reused after a deletion, within one run and across a
    /// save and reload (`ID-4`).
    pub fn issue_identifier(&mut self) -> Id {
        unimplemented!()
    }
}

/// How deformations touching one node combine.
///
/// One value today. A mode added in a later version must not change how a puppet
/// authored before it composes, which is why the field is written even when it
/// holds its default (`DET-6`, `docs/format.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompositionMode {
    /// Summation, the order specified now.
    Sum,
}

/// One entry in a puppet's tree.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: Id,
    /// The containing node, or [`Id::NONE`] for the root.
    pub parent: Id,
    pub name: String,
    /// The node's own local transform. A physics node's displacement is
    /// evaluated, never written back here (`PHYS-2`, out of scope in S2).
    pub transform: Transform,
    /// What this node is, and with it what it may contain.
    pub body: NodeBody,
    /// Marked, namespaced values the core format does not define.
    pub extensions: Extensions,
}

/// The four kinds of node, each with what only that kind carries.
///
/// Closed, by construction: a kind this build does not know never reaches the
/// tree, because an unknown kind arrives only in a file declaring a later version
/// and is preserved opaquely instead (`EXT-7`).
#[derive(Clone, Debug, PartialEq)]
pub enum NodeBody {
    /// Draws a mesh with a texture. A leaf.
    Part(Part),
    /// Deforms its children with its own mesh.
    MeshGroup(MeshGroup),
    /// Renders its children to a buffer so blending and masking apply to the
    /// group. Carries nothing of its own.
    Composite,
    /// Simulated motion anchored in the tree. Contains only physics.
    Physics(PhysicsNode),
}

impl NodeBody {
    /// Which kind this is, for the containment table ([`crate::may_contain`]).
    pub fn kind(&self) -> NodeKind {
        unimplemented!()
    }
}

/// A node that draws a mesh with a texture on it.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub mesh: Mesh,
    /// The texture this part **references**. Never owns one.
    pub texture: Id,
    /// Stored on this part's reference to that texture, per axis, so two parts
    /// may share one drawing with one of them reversed (`FLIP-1`, `FLIP-2`).
    pub texture_flip: TextureFlip,
    /// The part that restricts where this one draws, or [`Id::NONE`].
    pub mask: Id,
    /// Where this part's artwork came from. Never a path, a username or a
    /// machine name, and never resolved (`FILE-5`, `CANON-2`, `EXEC-1`).
    pub provenance: Option<Provenance>,
    /// A permanent part of the rig, not a bulk edit (`CHAIN-1`, `CHAIN-2`).
    pub chain_deformer: Option<ChainDeformer>,
}

/// A node that deforms its children with its own mesh.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshGroup {
    pub mesh: Mesh,
}

/// Simulated motion anchored in the tree.
///
/// One node is one anchor and one bob; a chain is nesting, so every constant is
/// per node (`docs/format.md` — *A physics node*). The anchor is this node's own
/// origin after its ancestors' transforms, so **there is no stored reference to
/// anything** for it to find that point by (`PHYS-1`).
///
/// Nothing simulated is ever stored here (`PHYS-8`).
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsNode {
    /// Rest distance from the anchor to the bob, in puppet pixels. Above zero.
    pub length: f64,
    /// Multiplier on the puppet's gravity; zero turns it off for this node
    /// alone. At or above zero.
    pub gravity: f64,
    /// Resonant frequency of the restoring spring, in Hz. Above zero.
    pub frequency: f64,
    /// Damping of swinging motion. At or above zero.
    pub angle_damping: f64,
    /// Damping of stretch. At or above zero.
    pub length_damping: f64,
    /// The one parameter this node drives, or [`Id::NONE`] for none. Driving
    /// nothing is ordinary and never a defect (`PHYS-4`).
    pub output: Id,
    /// Per axis of that parameter, which simulated quantity drives it.
    pub output_source: Vec<OutputSource>,
    /// Per axis of that parameter, the gain from that quantity onto it.
    pub output_scale: Vec<f64>,
}

/// Which simulated quantity drives one axis of a physics node's parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutputSource {
    Angle,
    Length,
    X,
    Y,
}

/// A node's own placement, before any binding or simulation touches it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translation: [f64; 2],
    pub rotation: f64,
    pub scale: [f64; 2],
}

/// The geometry a part or a mesh group carries.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<[f64; 2]>,
    /// In the texture's **authored** pixel space, not the cropped bitmap's
    /// (`docs/format.md` — *A part's mesh*), which is what makes cropping move
    /// nothing (`TEX-4`).
    pub uvs: Vec<[f64; 2]>,
    pub triangles: Vec<[u32; 3]>,
}

/// Whether a part samples its texture reversed, per axis.
///
/// The two axes are independent: one flag standing for both cannot express a
/// part reversed top to bottom (`FLIP-2`). Setting one creates nothing —
/// a texture flip is not a mirror (`FLIP-3`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureFlip {
    pub left_right: bool,
    pub top_bottom: bool,
}

/// What a part records about where its artwork came from.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Provenance {
    /// Taken from the source document where it provides one.
    pub source_id: String,
    /// Of the imported pixels.
    pub content_hash: String,
}

/// A sequence of linked joints deforming geometry along it.
#[derive(Clone, Debug, PartialEq)]
pub struct ChainDeformer {
    pub id: Id,
    pub joints: Vec<Joint>,
}

/// One joint of a chain deformer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Joint {
    pub id: Id,
    pub position: [f64; 2],
}

/// A puppet's own named axis with keypoints along it.
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub id: Id,
    pub name: String,
    /// One entry per dimension. A one-dimensional parameter declares one
    /// (`POL-1`).
    pub axes: Vec<Axis>,
    /// Ordered by position, within the declared range, with exactly one neutral
    /// (`VAL-4`, `VAL-15`).
    pub keypoints: Vec<Keypoint>,
}

/// One dimension of a parameter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axis {
    /// What the axis spans. Not its polarity.
    pub range: [f64; 2],
    /// Declared per axis, never inferred from an absence (`VAL-17`, `VAL-21`).
    pub polarity: AxisPolarity,
}

/// What a parameter declares about an axis — what reversing it would mean.
///
/// Three values, and the third is a value that must be **written**, never an
/// absence to be read as one (`VAL-21`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxisPolarity {
    /// The negative half mirrors the positive half.
    Bilateral,
    /// The axis has no mirror sense.
    Unipolar,
    /// Nobody has decided. Valid, and the value an import records rather than
    /// guessing.
    Unknown,
}

/// A position on a parameter's axes at which binding values are stored.
#[derive(Clone, Debug, PartialEq)]
pub struct Keypoint {
    pub id: Id,
    /// One coordinate per axis of the parameter.
    pub position: Vec<f64>,
    /// The authored resting value. Exactly one keypoint per parameter carries it
    /// (`VAL-15`).
    pub neutral: bool,
}

/// A mapping from a parameter to something that changes with it.
#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub id: Id,
    pub kind: BindingKind,
    /// The parameter this binding reads.
    pub parameter: Id,
    /// The node this binding drives.
    pub target: Id,
    /// One entry per keypoint of the parameter.
    pub values: BindingValues,
    /// Written only when set. Authored data kept although the mesh it applies to
    /// no longer matches it — a state, never a validation failure (`VAL-9`,
    /// `ART-5`). The mark must agree with the value counts in **both**
    /// directions (`VAL-5`).
    pub stale: bool,
}

/// Which of the two things a binding drives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BindingKind {
    /// Moves a whole node.
    Transform,
    /// Per-vertex offsets on a mesh.
    Deformation,
}

/// A binding's stored values, one entry per keypoint.
#[derive(Clone, Debug, PartialEq)]
pub enum BindingValues {
    /// One transform per keypoint.
    Transform(Vec<Transform>),
    /// Per keypoint, one offset per vertex of the target's mesh.
    Deformation(Vec<Vec<[f64; 2]>>),
}

impl BindingValues {
    /// How many keypoints' worth of values are stored. The count `VAL-5`
    /// compares against the parameter's keypoints.
    pub fn len(&self) -> usize {
        unimplemented!()
    }

    /// Whether no values are stored at all.
    pub fn is_empty(&self) -> bool {
        unimplemented!()
    }
}

/// Parameter values driven over time, composed of tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    pub id: Id,
    pub name: String,
    /// At most one track per parameter (`VAL-27`). A list rather than a map
    /// keyed by parameter, so the duplicate is expressible and can be rejected.
    pub tracks: Vec<Track>,
}

/// The part of an animation that drives exactly one parameter.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub id: Id,
    /// The one parameter this track drives (`TRACK-1`).
    pub parameter: Id,
    /// At or above zero (`VAL-26`).
    pub blend_duration: f64,
    /// Ordered in time, each value within the parameter's declared range
    /// (`VAL-26`).
    pub keyframes: Vec<Keyframe>,
}

/// One authored value of a track at one time.
#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe {
    pub time: f64,
    /// One coordinate per axis of the parameter.
    pub value: Vec<f64>,
}

/// A puppet's assignment of a channel to one of its parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelMapping {
    pub id: Id,
    /// A channel is named, not identified: it is supplied by a source the puppet
    /// has never met. Outside what [I3](../../../docs/invariants.md) binds.
    pub channel: String,
    pub parameter: Id,
    /// Optional, and empty in almost every puppet.
    pub source: String,
    /// The rigger's shaping, defined over the normalised channel.
    pub response_curve: ResponseCurve,
}

/// The authored shaping a channel mapping applies.
#[derive(Clone, Debug, PartialEq)]
pub struct ResponseCurve {
    pub points: Vec<[f64; 2]>,
}

/// An image, an entity in its own right, with its payload.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureRecord {
    pub id: Id,
    pub name: String,
    /// The dimensions of the artwork as authored, which with [`crop`] lets the
    /// runtime reconstruct the space a mesh's uvs are in (`TEX-4`).
    ///
    /// [`crop`]: TextureRecord::crop
    pub authored_size: [u32; 2],
    /// `[x, y, width, height]` of the content within the authored space.
    pub crop: [u32; 4],
    /// The stored bytes, exactly as the archive carries them. A payload that
    /// will not decode is written back byte for byte rather than re-encoded
    /// (`VAL-10`).
    pub payload: Vec<u8>,
}

/// Marked, namespaced values the core format does not define, on one object.
///
/// Preserved untouched, never executed ([I1](../../../docs/invariants.md)). An
/// entry with no declared version or no human-readable name is invalid
/// (`EXT-3`), and unmarked data outside this is invalid rather than an extension
/// (`EXT-2`, `VAL-8`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Extensions {
    /// Ordered by namespace, which is the canonical position — keyed on the
    /// marker, which a build can read without understanding the contents
    /// (`EXT-4`).
    pub entries: Vec<ExtensionEntry>,
}

/// One extension's data on one object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtensionEntry {
    /// The extension's own identifier, which is the key under `x`.
    pub namespace: String,
    /// Human-readable, and what a user without the extension is shown
    /// (`MSG-1`).
    pub name: String,
    /// The version this data expects.
    pub version: String,
    /// The `data` object's bytes, **verbatim as they arrived**. Never reparsed
    /// through our own number formatter and never re-emitted, because values we
    /// do not understand would be damaged silently (`EXT-4`, `EXT-5`, `EXT-9`).
    pub data: Vec<u8>,
}
