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
    /// Everything a file declaring a **later minor version** carried that this
    /// build has no code for, kept so that it survives a save untouched
    /// (`VER-1`, `VER-6`, `EXT-7`, `TOOL-4`).
    ///
    /// Empty for a file at this version or an earlier one, where the same
    /// material is invalid rather than preserved
    /// ([`crate::Rule::UnmarkedData`], `EXT-8`).
    pub preserved: Preserved,
}

/// Material a later-version file carried that this build has no code for.
///
/// `docs/format.md`: *the loader keeps the parsed source alongside its typed
/// structures rather than discarding it. That is a cost paid on every load, and
/// it is the mechanism the whole preserve-what-you-do-not-understand rule rests
/// on.* This is that parsed source, reduced to the parts the typed structures
/// have nowhere to put.
///
/// It hangs off [`Puppet`] rather than off each entity **because the acceptance
/// suite constructs [`Node`], [`Part`], [`Parameter`] and their neighbours by
/// struct literal**, so a field added to any of them would force an edit to a
/// test. Keying on the owner's identifier costs nothing by comparison: every
/// entity a puppet contains carries one, uniquely ([I3](../../../docs/invariants.md)).
///
/// Contents are verbatim and position is canonical, which is the pair `EXT-4`
/// asserts: the bytes are what arrived, and where they are written is set by the
/// canonical ordering rather than by where they were found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Preserved {
    /// Members of an object that the core format does not define, located by the
    /// identifier of the entity that carried them — the puppet's own identifier
    /// for the puppet object itself. Ordered by owner, then by name.
    pub members: Vec<PreservedMembers>,
    /// Whole node entries of a kind this build does not know, in the order they
    /// arrived. **Not** members of [`Puppet::nodes`]: an unknown kind is opaque
    /// material rather than a reported entity (`EXT-7`).
    pub opaque_nodes: Vec<OpaqueNode>,
}

impl Preserved {
    /// Whether nothing was preserved at all, which is the ordinary case.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty() && self.opaque_nodes.is_empty()
    }

    /// The members preserved at one place inside one entity.
    ///
    /// `path` is empty for the entity's own object — see
    /// [`PreservedMembers::path`].
    pub fn at(&self, owner: Id, path: &[&str]) -> &[(String, Vec<u8>)] {
        self.members
            .iter()
            .find(|entry| {
                entry.owner == owner
                    && entry.path.len() == path.len()
                    && entry.path.iter().zip(path).all(|(held, want)| held == want)
            })
            .map(|entry| entry.members.as_slice())
            .unwrap_or_default()
    }
}

/// The members one object inside one entity carried that the core format does
/// not define.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreservedMembers {
    /// The entity that carried them — the one whose identifier the stored
    /// object declares, which is what makes this survive the entity being
    /// reordered ([I3](../../../docs/invariants.md)).
    pub owner: Id,
    /// Where inside that entity's object they sat: empty for the object
    /// itself, `["transform"]` for its transform, `["axes", "1"]` for its
    /// second axis. A step is a member name, or an array position spelled in
    /// decimal — which of the two is never in doubt, because the shape at each
    /// step is one the core format defines.
    ///
    /// Only two arrays here are addressed by position, `axes` and `keyframes`,
    /// and both are positional in the format already. Everything else a puppet
    /// holds carries an identifier and is addressed by [`PreservedMembers::owner`].
    pub path: Vec<String>,
    /// `(name, the value's canonical JSON text)`, ordered by name. The text is
    /// never reparsed through our own number formatter, because values we by
    /// definition do not understand would be damaged silently (`EXT-9`).
    pub members: Vec<(String, Vec<u8>)>,
}

/// One node entry of a kind this build does not know.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpaqueNode {
    /// The identifier the entry carries. Read **only** to place the entry back
    /// in the canonical order, never to resolve anything: nothing in the puppet
    /// may reference a node this build cannot interpret.
    pub id: Id,
    /// The parent the entry names, read for the same reason and no other.
    pub parent: Id,
    /// The whole entry, as canonical JSON text.
    pub source: Vec<u8>,
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
        let mut out = vec![self.id];
        for node in &self.nodes {
            out.push(node.id);
            if let NodeBody::Part(part) = &node.body
                && let Some(chain) = &part.chain_deformer
            {
                out.push(chain.id);
                out.extend(chain.joints.iter().map(|joint| joint.id));
            }
        }
        out.extend(self.textures.iter().map(|texture| texture.id));
        for parameter in &self.parameters {
            out.push(parameter.id);
            out.extend(parameter.keypoints.iter().map(|keypoint| keypoint.id));
        }
        out.extend(self.bindings.iter().map(|binding| binding.id));
        for animation in &self.animations {
            out.push(animation.id);
            out.extend(animation.tracks.iter().map(|track| track.id));
        }
        out.extend(self.channel_mappings.iter().map(|mapping| mapping.id));
        out
    }

    /// Issues an identifier that has never been issued for this puppet.
    ///
    /// Takes the next value from [`Puppet::high_water_mark`] and advances it, so
    /// that no identifier is reused after a deletion, within one run and across a
    /// save and reload (`ID-4`).
    ///
    /// The mark is the **highest value ever issued**, not a count of what the
    /// puppet holds: advancing it before reading is what makes a deletion
    /// invisible here, and `docs/format.md` requires a writer to take it from
    /// the file it read rather than from what it found. [`Id::NONE`] is
    /// therefore never issued, because the first identifier a puppet issues is
    /// one.
    pub fn issue_identifier(&mut self) -> Id {
        self.high_water_mark = self
            .high_water_mark
            .checked_add(1)
            .expect("a puppet has not issued 2^64 identifiers");
        Id(self.high_water_mark)
    }
}

/// How deformations touching one node combine.
///
/// One value today. A mode added in a later version must not change how a puppet
/// authored before it composes, which is why the field is written even when it
/// holds its default (`DET-6`, `docs/format.md`).
///
/// **Open**, per *An unfamiliar value in a field this build knows* — see
/// [`AxisPolarity::Unfamiliar`] for what that variant means and when it is
/// legal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CompositionMode {
    /// Summation, the order specified now.
    Sum,
    /// A mode this build has never heard of, carrying its token verbatim.
    Unfamiliar(String),
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
        match self {
            NodeBody::Part(_) => NodeKind::Part,
            NodeBody::MeshGroup(_) => NodeKind::MeshGroup,
            NodeBody::Composite => NodeKind::Composite,
            NodeBody::Physics(_) => NodeKind::Physics,
        }
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
///
/// **Open**, per *An unfamiliar value in a field this build knows* — see
/// [`AxisPolarity::Unfamiliar`]. An axis whose source is unfamiliar is driven by
/// nothing: it does not fall back to `angle`, because guessing is the one thing
/// a build with no code for a value must not do (`docs/format.md`, `EXT-7`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OutputSource {
    Angle,
    Length,
    X,
    Y,
    /// A quantity this build has never heard of, carrying its token verbatim.
    Unfamiliar(String),
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
///
/// Not [`Copy`], because [`AxisPolarity`] is not: an unfamiliar polarity carries
/// its token verbatim and a token is a `String`.
#[derive(Clone, Debug, PartialEq)]
pub struct Axis {
    /// What the axis spans. Not its polarity.
    pub range: [f64; 2],
    /// Declared per axis, never inferred from an absence (`VAL-17`, `VAL-21`).
    pub polarity: AxisPolarity,
}

/// What a parameter declares about an axis — what reversing it would mean.
///
/// Three values the format defines, and the third is a value that must be
/// **written**, never an absence to be read as one (`VAL-21`).
///
/// # The fourth inhabitant
///
/// `docs/format.md`, *An unfamiliar value in a field this build knows*: every
/// enumerated field the core format defines has one more inhabitant than it
/// lists — the unfamiliar value, carrying the token verbatim. A closed enum
/// would force a reader to choose between refusing a file a later build wrote
/// and silently substituting a value nobody asked for.
///
/// [`AxisPolarity::Unknown`] and [`AxisPolarity::Unfamiliar`] are **different
/// things** and the pair is easy to conflate. *Unknown* is a value this format
/// defines, meaning the rigger has not decided; *unfamiliar* is a token this
/// build has never heard of, which a later minor version defined and this one
/// carries through without interpreting.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AxisPolarity {
    /// The negative half mirrors the positive half.
    Bilateral,
    /// The axis has no mirror sense.
    Unipolar,
    /// Nobody has decided. Valid, and the value an import records rather than
    /// guessing.
    Unknown,
    /// A value this build has never heard of, carrying the token verbatim.
    ///
    /// **Invalid at [`crate::FormatVersion::CURRENT`]** — a writer at this
    /// version had the whole list and used something else, so a typo and an
    /// addition are indistinguishable ([`crate::Rule::EnumeratedValueKnown`]).
    /// In a file declaring a **later** minor version it loads, is inert
    /// wherever it would have been interpreted, and is written back verbatim.
    ///
    /// The gate is the **declared version**, never recognisability, which is
    /// the same gate extension data under `x` carries.
    Unfamiliar(String),
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
        match self {
            BindingValues::Transform(values) => values.len(),
            BindingValues::Deformation(values) => values.len(),
        }
    }

    /// Whether no values are stored at all.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

#[cfg(test)]
mod tests {
    use super::*;

    fn transform() -> Transform {
        Transform {
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        }
    }

    fn mesh() -> Mesh {
        Mesh {
            vertices: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            triangles: vec![[0, 1, 2]],
        }
    }

    /// A puppet carrying one of every piece *What a puppet is* lists, so that
    /// [`Puppet::identifiers`] is asserted against a subject that has something
    /// of each kind to forget.
    fn one_of_everything() -> Puppet {
        Puppet {
            id: Id(1),
            name: "subject".to_owned(),
            high_water_mark: 20,
            root: Id(2),
            gravity: 1.0,
            composition_mode: CompositionMode::Sum,
            nodes: vec![
                Node {
                    id: Id(2),
                    parent: Id::NONE,
                    name: "root".to_owned(),
                    transform: transform(),
                    body: NodeBody::Composite,
                    extensions: Extensions::default(),
                },
                Node {
                    id: Id(3),
                    parent: Id(2),
                    name: "group".to_owned(),
                    transform: transform(),
                    body: NodeBody::MeshGroup(MeshGroup { mesh: mesh() }),
                    extensions: Extensions::default(),
                },
                Node {
                    id: Id(4),
                    parent: Id(3),
                    name: "part".to_owned(),
                    transform: transform(),
                    body: NodeBody::Part(Part {
                        mesh: mesh(),
                        texture: Id(7),
                        texture_flip: TextureFlip::default(),
                        mask: Id::NONE,
                        provenance: None,
                        chain_deformer: Some(ChainDeformer {
                            id: Id(5),
                            joints: vec![Joint {
                                id: Id(6),
                                position: [0.0, 0.0],
                            }],
                        }),
                    }),
                    extensions: Extensions::default(),
                },
                Node {
                    id: Id(20),
                    parent: Id(3),
                    name: "strand".to_owned(),
                    transform: transform(),
                    body: NodeBody::Physics(PhysicsNode {
                        length: 100.0,
                        gravity: 1.0,
                        frequency: 1.0,
                        angle_damping: 0.5,
                        length_damping: 0.5,
                        output: Id::NONE,
                        output_source: Vec::new(),
                        output_scale: Vec::new(),
                    }),
                    extensions: Extensions::default(),
                },
            ],
            parameters: vec![Parameter {
                id: Id(8),
                name: "parameter".to_owned(),
                axes: vec![Axis {
                    range: [-1.0, 1.0],
                    polarity: AxisPolarity::Bilateral,
                }],
                keypoints: vec![Keypoint {
                    id: Id(9),
                    position: vec![0.0],
                    neutral: true,
                }],
            }],
            bindings: vec![Binding {
                id: Id(10),
                kind: BindingKind::Transform,
                parameter: Id(8),
                target: Id(4),
                values: BindingValues::Transform(vec![transform()]),
                stale: false,
            }],
            animations: vec![Animation {
                id: Id(11),
                name: "animation".to_owned(),
                tracks: vec![Track {
                    id: Id(12),
                    parameter: Id(8),
                    blend_duration: 0.0,
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: vec![0.0],
                    }],
                }],
            }],
            channel_mappings: vec![ChannelMapping {
                id: Id(13),
                channel: "head_turn".to_owned(),
                parameter: Id(8),
                source: String::new(),
                response_curve: ResponseCurve {
                    points: vec![[0.0, 0.0], [1.0, 1.0]],
                },
            }],
            textures: vec![TextureRecord {
                id: Id(7),
                name: "face".to_owned(),
                authored_size: [16, 16],
                crop: [0, 0, 16, 16],
                payload: Vec::new(),
            }],
            extensions: Extensions::default(),
            preserved: Preserved::default(),
        }
    }

    /// `ID-1` and `ID-8`: every piece is enumerated, the chain deformer and its
    /// joints and each animation's tracks included. A kind dropped from
    /// [`Puppet::identifiers`] is a kind no reload can be checked against.
    #[test]
    fn identifiers_enumerate_every_piece() {
        let subject = one_of_everything();
        let mut found = subject.identifiers();
        found.sort();
        assert_eq!(
            found,
            (1..=13)
                .chain(std::iter::once(20))
                .map(Id)
                .collect::<Vec<_>>(),
            "the puppet, every node, the chain deformer and its joint, the \
             texture, the parameter and its keypoint, the binding, the \
             animation and its track, and the channel mapping"
        );
    }

    #[test]
    fn identifiers_include_the_puppets_own() {
        let subject = one_of_everything();
        assert!(subject.identifiers().contains(&subject.id));
    }

    /// `ID-4`: the mark advances, so a deleted identifier is never reissued.
    #[test]
    fn issuing_never_repeats_and_never_goes_back() {
        let mut subject = one_of_everything();
        let before = subject.identifiers();
        let first = subject.issue_identifier();
        let second = subject.issue_identifier();

        assert_ne!(first, second);
        assert!(first < second, "the mark only ever advances");
        assert!(!before.contains(&first), "{first:?} is in use already");
        assert!(!before.contains(&second), "{second:?} is in use already");
        assert_eq!(subject.high_water_mark, second.0);
    }

    /// Deleting the whole puppet and then issuing must still not hand back an
    /// identifier that was in use, which is what makes the mark a mark rather
    /// than a count.
    #[test]
    fn a_deletion_does_not_lower_the_mark() {
        let mut subject = one_of_everything();
        let retired: Vec<Id> = subject.identifiers();
        subject.nodes.clear();
        subject.parameters.clear();
        subject.bindings.clear();
        subject.animations.clear();
        subject.channel_mappings.clear();
        subject.textures.clear();

        for _ in 0..8 {
            let issued = subject.issue_identifier();
            assert!(!retired.contains(&issued), "{issued:?} came back");
        }
    }

    /// The reserved zero means *no reference* and is never handed out, even by a
    /// puppet that has issued nothing.
    #[test]
    fn the_reserved_zero_is_never_issued() {
        let mut fresh = one_of_everything();
        fresh.high_water_mark = 0;
        assert_ne!(fresh.issue_identifier(), Id::NONE);
    }

    #[test]
    fn a_body_reports_its_kind() {
        let subject = one_of_everything();
        let kinds: Vec<NodeKind> = subject.nodes.iter().map(|n| n.body.kind()).collect();
        assert_eq!(
            kinds,
            vec![
                NodeKind::Composite,
                NodeKind::MeshGroup,
                NodeKind::Part,
                NodeKind::Physics
            ]
        );
    }

    /// `VAL-5` compares this count against the parameter's keypoints, so it
    /// counts keypoints' worth of values rather than vertices.
    #[test]
    fn binding_values_count_keypoints() {
        let transforms = BindingValues::Transform(vec![transform(), transform()]);
        assert_eq!(transforms.len(), 2);
        assert!(!transforms.is_empty());

        let deformations = BindingValues::Deformation(vec![
            vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]],
            vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]],
        ]);
        assert_eq!(
            deformations.len(),
            2,
            "two keypoints' worth of three vertices each, not six"
        );

        assert!(BindingValues::Transform(Vec::new()).is_empty());
        assert!(BindingValues::Deformation(Vec::new()).is_empty());
    }

    /// The unfamiliar inhabitant is a value, distinct from every value the
    /// format defines — `AxisPolarity::Unknown` most of all, which is a decision
    /// the rigger declined to make rather than a token we cannot read.
    #[test]
    fn an_unfamiliar_value_is_not_any_value_the_format_defines() {
        let unfamiliar = AxisPolarity::Unfamiliar("radial".to_owned());
        for defined in [
            AxisPolarity::Bilateral,
            AxisPolarity::Unipolar,
            AxisPolarity::Unknown,
        ] {
            assert_ne!(unfamiliar, defined);
        }
        assert_ne!(
            AxisPolarity::Unfamiliar("radial".to_owned()),
            AxisPolarity::Unfamiliar("torque".to_owned()),
            "the token is what distinguishes one from another"
        );
        assert_ne!(
            OutputSource::Unfamiliar("torque".to_owned()),
            OutputSource::Angle
        );
        assert_ne!(
            CompositionMode::Unfamiliar("multiply".to_owned()),
            CompositionMode::Sum
        );
    }
}
