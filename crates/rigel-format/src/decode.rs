//! The parse: `puppet.json` into a [`Puppet`], preserving what it cannot read.
//!
//! # Two failures, deliberately kept apart
//!
//! A **shape** this build cannot read at all — `nodes` that is not an array, an
//! `id` that is not a number — is [`Rejection::Malformed`]. There is no rule to
//! name and often no identifier to blame, because the entity never came into
//! existence.
//!
//! A **rule** the file breaks is a [`Violation`]. Three families of them are
//! reachable only from here, because they are facts about the source text that a
//! [`Puppet`] cannot hold:
//!
//! | Rule | Why it cannot wait for the validator |
//! |---|---|
//! | [`Rule::UnmarkedData`] | A key outside `x` that the core format does not define. Once parsed, the key is either in a typed field or gone |
//! | [`Rule::EnumeratedValueKnown`] | Gated on the **declared version**, and a `Puppet` carries no version — deliberately, because a save decides whether it is upgrading by reading the file it replaces |
//! | [`Rule::AxisPolarityDeclared`], [`Rule::ExtensionDeclaresVersion`], [`Rule::ExtensionDeclaresName`] | Each is about a member that is **absent**, and an absence has no typed inhabitant to be found later (`VAL-21`) |
//!
//! Everything else — acyclicity, uniqueness, containment, ranges, finiteness —
//! is [`crate::validate`]'s, over the puppet this module produces.
//!
//! # What *licensed* means
//!
//! A file declaring a **later minor version** licenses material this build has
//! no code for: unmarked members, node kinds it does not know, and unfamiliar
//! values in enumerated fields it does. The same material at this version or an
//! earlier one is a defect, because a writer at this version had the whole list
//! and used something else (`docs/format.md` — *An unfamiliar value in a field
//! this build knows*, `EXT-8`).

use crate::json::Json;
use crate::limits::LimitKind;
use crate::load::{ExtensionNotice, Rejection};
use crate::puppet::{
    Animation, Axis, AxisPolarity, Binding, BindingKind, BindingValues, ChainDeformer,
    ChannelMapping, CompositionMode, ExtensionEntry, Extensions, Id, Joint, Keyframe, Keypoint,
    Mesh, MeshGroup, Node, NodeBody, OpaqueNode, OutputSource, Parameter, Part, Preserved,
    PreservedMembers, Provenance, Puppet, ResponseCurve, TextureFlip, TextureRecord, Track,
    Transform,
};
use crate::validate::{Rule, Violation};

/// The most violations one rejection carries.
///
/// `VAL-12` asks for bounded output, and a list that grows with the file is a
/// second denial-of-service route through the error path. A file breaking more
/// rules than this is refused on the first of them that fit.
const MAX_VIOLATIONS: usize = 64;

/// What the parse produced.
pub(crate) struct Decoded {
    pub puppet: Puppet,
    /// The rules broken that only the source text could show.
    pub violations: Vec<Violation>,
    /// Bindings carrying the stale mark, for the load report.
    pub stale_bindings: Vec<Id>,
    /// Every extension namespace in the file, in namespace order. This build
    /// implements none, so every one of them is unrecognised.
    pub unrecognised_extensions: Vec<ExtensionNotice>,
}

/// Parses the structure entry into a puppet.
///
/// `declared` is the version `rigel2d.json` carried. A minor later than this
/// build's **licenses** material it has no code for, and is kept on the puppet
/// so the writer declares it again rather than demoting the file to this
/// version and invalidating what that declaration licensed.
pub(crate) fn decode(
    document: &Json,
    declared: crate::version::FormatVersion,
) -> Result<Decoded, Rejection> {
    let licensed = declared.minor > crate::version::FormatVersion::CURRENT.minor;
    let mut decoder = Decoder {
        licensed,
        violations: Vec::new(),
        preserved: Preserved::default(),
        stale_bindings: Vec::new(),
        notices: Vec::new(),
    };
    let puppet = decoder.puppet(document)?;
    decoder
        .preserved
        .members
        .sort_by(|a, b| a.owner.cmp(&b.owner).then_with(|| a.path.cmp(&b.path)));
    let mut puppet = puppet;
    puppet.preserved = std::mem::take(&mut decoder.preserved);
    puppet.declared_version = licensed.then_some(declared);
    decoder
        .notices
        .sort_by(|a, b| a.namespace.cmp(&b.namespace));
    decoder.notices.dedup_by(|a, b| a.namespace == b.namespace);
    Ok(Decoded {
        puppet,
        violations: decoder.violations,
        stale_bindings: decoder.stale_bindings,
        unrecognised_extensions: decoder.notices,
    })
}

/// A member path inside one entity's object, for [`Preserved`].
type Path = Vec<String>;

fn at(path: &[&str]) -> Path {
    path.iter().map(|step| (*step).to_owned()).collect()
}

struct Decoder {
    licensed: bool,
    violations: Vec<Violation>,
    preserved: Preserved,
    stale_bindings: Vec<Id>,
    notices: Vec<ExtensionNotice>,
}

// ------------------------------------------------------------ the key sets
//
// These are what [`Rule::UnmarkedData`] is judged against, and so they are also
// what the committed JSON Schema must list — `crates/rigel-format/schema/`.
// They are `pub(crate)` rather than private for exactly that: a unit test in
// `crate::validate` holds the schema's member lists against them, so a member
// added to one and not the other fails a test rather than waiting to be found
// by whoever runs the schema next. Nothing outside this crate reads them.

/// The members the core format defines on the puppet object.
pub(crate) const PUPPET_KEYS: &[&str] = &[
    "id",
    "name",
    "high_water_mark",
    "root",
    "gravity",
    "composition_mode",
    "nodes",
    "parameters",
    "bindings",
    "animations",
    "channel_mappings",
    "textures",
];
/// What every node carries, whatever its kind.
pub(crate) const NODE_KEYS: &[&str] = &["id", "parent", "name", "transform", "kind"];
pub(crate) const PART_KEYS: &[&str] = &[
    "mesh",
    "texture",
    "texture_flip",
    "mask",
    "provenance",
    "chain_deformer",
];
pub(crate) const MESH_GROUP_KEYS: &[&str] = &["mesh"];
pub(crate) const PHYSICS_KEYS: &[&str] = &[
    "length",
    "gravity",
    "frequency",
    "angle_damping",
    "length_damping",
    "output",
    "output_source",
    "output_scale",
];
pub(crate) const TRANSFORM_KEYS: &[&str] = &["translation", "rotation", "scale"];
pub(crate) const MESH_KEYS: &[&str] = &["vertices", "uvs", "triangles"];
pub(crate) const FLIP_KEYS: &[&str] = &["left_right", "top_bottom"];
pub(crate) const PROVENANCE_KEYS: &[&str] = &["source_id", "content_hash"];
pub(crate) const CHAIN_KEYS: &[&str] = &["id", "joints"];
pub(crate) const JOINT_KEYS: &[&str] = &["id", "position"];
pub(crate) const PARAMETER_KEYS: &[&str] = &["id", "name", "axes", "keypoints"];
pub(crate) const AXIS_KEYS: &[&str] = &["range", "polarity"];
pub(crate) const KEYPOINT_KEYS: &[&str] = &["id", "position", "neutral"];
pub(crate) const BINDING_KEYS: &[&str] = &["id", "kind", "parameter", "target", "values", "stale"];
pub(crate) const ANIMATION_KEYS: &[&str] = &["id", "name", "tracks"];
pub(crate) const TRACK_KEYS: &[&str] = &["id", "parameter", "blend_duration", "keyframes"];
pub(crate) const KEYFRAME_KEYS: &[&str] = &["time", "value"];
pub(crate) const MAPPING_KEYS: &[&str] =
    &["id", "channel", "parameter", "source", "response_curve"];
pub(crate) const CURVE_KEYS: &[&str] = &["points"];
pub(crate) const TEXTURE_KEYS: &[&str] = &["id", "name", "authored_size", "crop"];
pub(crate) const EXTENSION_KEYS: &[&str] = &["version", "name", "data"];

/// The node kinds this build has code for.
pub(crate) const KNOWN_NODE_KINDS: &[&str] = &["part", "mesh_group", "composite", "physics"];

// ------------------------------------------------------------- the helpers

fn malformed(detail: impl Into<String>) -> Rejection {
    Rejection::Malformed {
        detail: detail.into(),
    }
}

impl Decoder {
    fn violate(&mut self, rule: Rule, offenders: Vec<Id>, field: Option<&str>, detail: String) {
        if self.violations.len() >= MAX_VIOLATIONS {
            return;
        }
        self.violations.push(Violation {
            rule,
            offenders,
            field: field.map(str::to_owned),
            detail,
        });
    }

    fn object<'a>(&self, value: &'a Json, what: &str) -> Result<&'a [(String, Json)], Rejection> {
        value
            .as_object()
            .ok_or_else(|| malformed(format!("{what} is {}, not an object", value.kind())))
    }

    fn array<'a>(&self, value: &'a Json, what: &str) -> Result<&'a [Json], Rejection> {
        value
            .as_array()
            .ok_or_else(|| malformed(format!("{what} is {}, not an array", value.kind())))
    }

    fn number(&self, value: &Json, what: &str) -> Result<f64, Rejection> {
        value
            .as_f64()
            .ok_or_else(|| malformed(format!("{what} is {}, not a number", value.kind())))
    }

    fn flag(&self, value: &Json, what: &str) -> Result<bool, Rejection> {
        value
            .as_bool()
            .ok_or_else(|| malformed(format!("{what} is {}, not a boolean", value.kind())))
    }

    fn identifier(&self, value: &Json, what: &str) -> Result<Id, Rejection> {
        value
            .as_u64()
            .map(Id)
            .ok_or_else(|| malformed(format!("{what} is not an identifier")))
    }

    /// A string, refused against [`LimitKind::StringBytes`] before it is kept.
    fn text(&self, value: &Json, what: &str) -> Result<String, Rejection> {
        let text = value
            .as_str()
            .ok_or_else(|| malformed(format!("{what} is {}, not a string", value.kind())))?;
        crate::archive::within(LimitKind::StringBytes, text.len() as u64, what)?;
        Ok(text.to_owned())
    }

    fn required<'a>(
        &self,
        object: &'a [(String, Json)],
        key: &str,
        what: &str,
    ) -> Result<&'a Json, Rejection> {
        object
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
            .ok_or_else(|| malformed(format!("{what} carries no {key}")))
    }

    fn optional<'a>(&self, object: &'a [(String, Json)], key: &str) -> Option<&'a Json> {
        object
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    fn number_or(
        &self,
        object: &[(String, Json)],
        key: &str,
        default: f64,
    ) -> Result<f64, Rejection> {
        match self.optional(object, key) {
            Some(value) => self.number(value, key),
            None => Ok(default),
        }
    }

    fn flag_or(
        &self,
        object: &[(String, Json)],
        key: &str,
        default: bool,
    ) -> Result<bool, Rejection> {
        match self.optional(object, key) {
            Some(value) => self.flag(value, key),
            None => Ok(default),
        }
    }

    fn identifier_or_none(&self, object: &[(String, Json)], key: &str) -> Result<Id, Rejection> {
        match self.optional(object, key) {
            Some(value) => self.identifier(value, key),
            None => Ok(Id::NONE),
        }
    }

    /// `[x, y]`, the pair every coordinate in the format is.
    fn pair(&self, value: &Json, what: &str) -> Result<[f64; 2], Rejection> {
        let items = self.array(value, what)?;
        if items.len() != 2 {
            return Err(malformed(format!("{what} is not a pair")));
        }
        Ok([self.number(&items[0], what)?, self.number(&items[1], what)?])
    }

    fn dimensions(&self, value: &Json, what: &str) -> Result<[u32; 2], Rejection> {
        let items = self.array(value, what)?;
        if items.len() != 2 {
            return Err(malformed(format!("{what} is not a pair")));
        }
        let mut out = [0u32; 2];
        for (slot, item) in out.iter_mut().zip(items) {
            let found = item
                .as_u64()
                .ok_or_else(|| malformed(format!("{what} is not a whole number of pixels")))?;
            crate::archive::within(LimitKind::TextureDimension, found, what)?;
            *slot = u32::try_from(found)
                .map_err(|_| malformed(format!("{what} does not fit a pixel count")))?;
        }
        Ok(out)
    }

    // ------------------------------------------------- unknown members

    /// Records every member of `object` the core format does not define.
    ///
    /// Licensed, it is kept verbatim against the entity that carried it;
    /// unlicensed, it is [`Rule::UnmarkedData`] — without that rule a typo and
    /// an extension are indistinguishable and the validator can reject neither
    /// (`docs/format.md`, `EXT-2`).
    fn sweep(&mut self, owner: Id, path: &[&str], object: &[(String, Json)], known: &[&str]) {
        let mut kept: Vec<(String, Vec<u8>)> = Vec::new();
        for (name, value) in object {
            if known.contains(&name.as_str()) || name == "x" {
                continue;
            }
            if self.licensed {
                kept.push((name.clone(), value.to_canonical_bytes()));
            } else {
                self.violate(
                    Rule::UnmarkedData,
                    vec![owner],
                    Some(name),
                    format!("{name:?} is not a member the core format defines"),
                );
            }
        }
        if !kept.is_empty() {
            kept.sort_by(|a, b| a.0.cmp(&b.0));
            self.preserved.members.push(PreservedMembers {
                owner,
                path: at(path),
                members: kept,
            });
        }
    }

    /// Keeps a member this build has no typed slot for, at any declared version.
    ///
    /// Used for marked extension data on an object [`Extensions`] does not hang
    /// off, and for members inside an extension's own marker: both are legal
    /// wherever they appear, and dropping them would damage the file on save.
    fn keep(&mut self, owner: Id, path: &[&str], name: &str, value: &Json) {
        self.preserved.members.push(PreservedMembers {
            owner,
            path: at(path),
            members: vec![(name.to_owned(), value.to_canonical_bytes())],
        });
    }

    // --------------------------------------------------- extension data

    /// The `x` marker: namespaced values the core format does not define.
    ///
    /// Every namespace is reported as unrecognised, because this build
    /// implements none. Reporting is **all** that happens: declaring is not
    /// installing ([I6](../../../docs/invariants.md), `EXT-6`).
    fn extensions(
        &mut self,
        owner: Id,
        path: &[&str],
        object: &[(String, Json)],
    ) -> Result<Extensions, Rejection> {
        let Some(marker) = self.optional(object, "x") else {
            return Ok(Extensions::default());
        };
        let namespaces = self.object(marker, "x")?;
        let mut entries = Vec::with_capacity(namespaces.len());
        for (namespace, body) in namespaces {
            crate::archive::within(
                LimitKind::StringBytes,
                namespace.len() as u64,
                "an extension namespace",
            )?;
            let fields = self.object(body, "an extension's data")?;
            let version = match self.optional(fields, "version") {
                Some(value) => self.text(value, "an extension's version")?,
                None => {
                    self.violate(
                        Rule::ExtensionDeclaresVersion,
                        vec![owner],
                        Some(namespace),
                        format!("{namespace} declares no version"),
                    );
                    String::new()
                }
            };
            let name = match self.optional(fields, "name") {
                Some(value) => self.text(value, "an extension's name")?,
                None => {
                    self.violate(
                        Rule::ExtensionDeclaresName,
                        vec![owner],
                        Some(namespace),
                        format!("{namespace} declares no human-readable name"),
                    );
                    String::new()
                }
            };
            let data = self
                .required(fields, "data", &format!("the extension {namespace}"))?
                .to_canonical_bytes();
            crate::archive::within(
                LimitKind::ExtensionDataBytes,
                data.len() as u64,
                "extension data",
            )?;

            // Anything else inside the marker is somebody else's and stays.
            let mut inside = path.to_vec();
            inside.push("x");
            inside.push(namespace);
            for (member, value) in fields {
                if EXTENSION_KEYS.contains(&member.as_str()) {
                    continue;
                }
                self.keep(owner, &inside, member, value);
            }

            if !name.is_empty() && !version.is_empty() {
                self.notices.push(ExtensionNotice {
                    namespace: namespace.clone(),
                    name: name.clone(),
                    version: version.clone(),
                });
            }
            entries.push(ExtensionEntry {
                namespace: namespace.clone(),
                name,
                version,
                data,
            });
        }
        // Ordered by namespace, which is the canonical position: keyed on the
        // marker, which a build can read without understanding the contents
        // (`EXT-4`).
        entries.sort_by(|a, b| a.namespace.cmp(&b.namespace));
        Ok(Extensions { entries })
    }

    /// The same, for an object with no [`Extensions`] slot. The marker is kept
    /// verbatim instead, so it survives a save.
    fn extensions_preserved(
        &mut self,
        owner: Id,
        path: &[&str],
        object: &[(String, Json)],
    ) -> Result<(), Rejection> {
        if let Some(marker) = self.optional(object, "x") {
            let _ = self.extensions(owner, path, object)?;
            self.keep(owner, path, "x", marker);
        }
        Ok(())
    }

    // ------------------------------------------------ enumerated fields

    /// One value of an enumerated field the core format defines.
    ///
    /// Every such field has one more inhabitant than it lists — the unfamiliar
    /// value, carrying the token verbatim — and it is legal only in a file
    /// declaring a later minor version (`docs/format.md`, `VAL-7`, `VAL-20`).
    fn enumerated<T>(
        &mut self,
        owner: Id,
        field: &str,
        token: &str,
        known: &[(&str, T)],
        unfamiliar: impl FnOnce(String) -> T,
    ) -> T
    where
        T: Clone,
    {
        if let Some((_, value)) = known.iter().find(|(name, _)| *name == token) {
            return value.clone();
        }
        if !self.licensed {
            self.violate(
                Rule::EnumeratedValueKnown,
                vec![owner],
                Some(field),
                format!("{token:?} is not a value {field} takes at this version"),
            );
        }
        unfamiliar(token.to_owned())
    }

    // ------------------------------------------------------ the puppet

    fn puppet(&mut self, document: &Json) -> Result<Puppet, Rejection> {
        let object = self.object(document, "puppet.json")?;
        let id = self.identifier(
            self.required(object, "id", "the puppet")?,
            "the puppet's id",
        )?;
        let name = self.text(
            self.required(object, "name", "the puppet")?,
            "the puppet's name",
        )?;
        let high_water_mark = self
            .required(object, "high_water_mark", "the puppet")?
            .as_u64()
            .ok_or_else(|| malformed("the high-water mark is not a whole number"))?;
        let root = self.identifier(self.required(object, "root", "the puppet")?, "the root")?;
        let gravity = self.number_or(object, "gravity", 1.0)?;

        let composition_mode = match self.optional(object, "composition_mode") {
            Some(value) => {
                let token = self.text(value, "composition_mode")?;
                self.enumerated(
                    id,
                    "composition_mode",
                    &token,
                    &[("sum", CompositionMode::Sum)],
                    CompositionMode::Unfamiliar,
                )
            }
            None => CompositionMode::Sum,
        };

        let extensions = self.extensions(id, &[], object)?;
        self.sweep(id, &[], object, PUPPET_KEYS);

        let nodes = self.nodes(object)?;
        let parameters = self.parameters(object)?;
        let bindings = self.bindings(object)?;
        let animations = self.animations(object)?;
        let channel_mappings = self.channel_mappings(object)?;
        let textures = self.textures(object)?;

        Ok(Puppet {
            id,
            name,
            high_water_mark,
            root,
            gravity,
            composition_mode,
            nodes,
            parameters,
            bindings,
            animations,
            channel_mappings,
            textures,
            extensions,
            // Both filled by `decode` once the walk is done.
            declared_version: None,
            preserved: Preserved::default(),
        })
    }

    // -------------------------------------------------------- the tree

    fn nodes(&mut self, object: &[(String, Json)]) -> Result<Vec<Node>, Rejection> {
        let Some(value) = self.optional(object, "nodes") else {
            return Ok(Vec::new());
        };
        let items = self.array(value, "nodes")?;
        crate::archive::within(LimitKind::NodeCount, items.len() as u64, "nodes")?;

        let mut nodes = Vec::with_capacity(items.len());
        for item in items {
            let fields = self.object(item, "a node")?;
            let id = self.identifier(self.required(fields, "id", "a node")?, "a node's id")?;
            let kind = self.text(self.required(fields, "kind", "a node")?, "a node's kind")?;

            if !KNOWN_NODE_KINDS.contains(&kind.as_str()) {
                // A kind this build does not know is opaque material rather
                // than a tree entry: deleting it must change no reported
                // entity (`EXT-7`).
                if !self.licensed {
                    self.violate(
                        Rule::EnumeratedValueKnown,
                        vec![id],
                        Some("kind"),
                        format!("{kind:?} is not a node kind at this version"),
                    );
                    continue;
                }
                let parent = self.identifier_or_none(fields, "parent")?;
                self.preserved.opaque_nodes.push(OpaqueNode {
                    id,
                    parent,
                    source: item.to_canonical_bytes(),
                });
                continue;
            }

            nodes.push(self.node(id, &kind, fields)?);
        }
        self.refuse_deep_trees(&nodes)?;
        Ok(nodes)
    }

    fn node(&mut self, id: Id, kind: &str, fields: &[(String, Json)]) -> Result<Node, Rejection> {
        let parent = self.identifier_or_none(fields, "parent")?;
        let name = self.text(self.required(fields, "name", "a node")?, "a node's name")?;
        let transform = self.transform(id, &["transform"], fields, "transform")?;
        let extensions = self.extensions(id, &[], fields)?;

        let (body, kind_keys): (NodeBody, &[&str]) = match kind {
            "composite" => (NodeBody::Composite, &[]),
            "mesh_group" => (
                NodeBody::MeshGroup(MeshGroup {
                    mesh: self.mesh(id, fields)?,
                }),
                MESH_GROUP_KEYS,
            ),
            "part" => (NodeBody::Part(self.part(id, fields)?), PART_KEYS),
            "physics" => (NodeBody::Physics(self.physics(id, fields)?), PHYSICS_KEYS),
            other => return Err(malformed(format!("{other:?} is not a node kind"))),
        };

        let mut known: Vec<&str> = NODE_KEYS.to_vec();
        known.extend_from_slice(kind_keys);
        self.sweep(id, &[], fields, &known);

        Ok(Node {
            id,
            parent,
            name,
            transform,
            body,
            extensions,
        })
    }

    fn part(&mut self, id: Id, fields: &[(String, Json)]) -> Result<Part, Rejection> {
        let mesh = self.mesh(id, fields)?;
        let texture = self.identifier(
            self.required(fields, "texture", "a part")?,
            "a part's texture",
        )?;
        let mask = self.identifier_or_none(fields, "mask")?;

        let texture_flip = match self.optional(fields, "texture_flip") {
            Some(value) => {
                let flip = self.object(value, "texture_flip")?;
                self.extensions_preserved(id, &["texture_flip"], flip)?;
                self.sweep(id, &["texture_flip"], flip, FLIP_KEYS);
                TextureFlip {
                    left_right: self.flag_or(flip, "left_right", false)?,
                    top_bottom: self.flag_or(flip, "top_bottom", false)?,
                }
            }
            None => TextureFlip::default(),
        };

        let provenance = match self.optional(fields, "provenance") {
            Some(value) => {
                let record = self.object(value, "provenance")?;
                self.extensions_preserved(id, &["provenance"], record)?;
                self.sweep(id, &["provenance"], record, PROVENANCE_KEYS);
                Some(Provenance {
                    source_id: self.text(
                        self.required(record, "source_id", "provenance")?,
                        "a source identifier",
                    )?,
                    content_hash: self.text(
                        self.required(record, "content_hash", "provenance")?,
                        "a content hash",
                    )?,
                })
            }
            None => None,
        };

        let chain_deformer = match self.optional(fields, "chain_deformer") {
            Some(value) => Some(self.chain(value)?),
            None => None,
        };

        Ok(Part {
            mesh,
            texture,
            texture_flip,
            mask,
            provenance,
            chain_deformer,
        })
    }

    fn chain(&mut self, value: &Json) -> Result<ChainDeformer, Rejection> {
        let fields = self.object(value, "a chain deformer")?;
        let id = self.identifier(
            self.required(fields, "id", "a chain deformer")?,
            "a chain deformer's id",
        )?;
        self.extensions_preserved(id, &[], fields)?;
        self.sweep(id, &[], fields, CHAIN_KEYS);

        let items = self.array(
            self.required(fields, "joints", "a chain deformer")?,
            "a chain deformer's joints",
        )?;
        crate::archive::within(LimitKind::ChainJointCount, items.len() as u64, "joints")?;

        let mut joints = Vec::with_capacity(items.len());
        for item in items {
            let joint = self.object(item, "a joint")?;
            let joint_id =
                self.identifier(self.required(joint, "id", "a joint")?, "a joint's id")?;
            self.extensions_preserved(joint_id, &[], joint)?;
            self.sweep(joint_id, &[], joint, JOINT_KEYS);
            joints.push(Joint {
                id: joint_id,
                position: self.pair(
                    self.required(joint, "position", "a joint")?,
                    "a joint's position",
                )?,
            });
        }
        Ok(ChainDeformer { id, joints })
    }

    fn physics(
        &mut self,
        id: Id,
        fields: &[(String, Json)],
    ) -> Result<crate::puppet::PhysicsNode, Rejection> {
        // The defaults are `docs/format.md`'s table, *A physics node*.
        let output = self.identifier_or_none(fields, "output")?;
        let sources = match self.optional(fields, "output_source") {
            Some(value) => {
                let items = self.array(value, "output_source")?;
                crate::archive::within(
                    LimitKind::AxisCount,
                    items.len() as u64,
                    "a physics node's output sources",
                )?;
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    let token = self.text(item, "an output source")?;
                    out.push(self.enumerated(
                        id,
                        "output_source",
                        &token,
                        &[
                            ("angle", OutputSource::Angle),
                            ("length", OutputSource::Length),
                            ("x", OutputSource::X),
                            ("y", OutputSource::Y),
                        ],
                        OutputSource::Unfamiliar,
                    ));
                }
                out
            }
            None => Vec::new(),
        };
        let scales = match self.optional(fields, "output_scale") {
            Some(value) => {
                let items = self.array(value, "output_scale")?;
                crate::archive::within(
                    LimitKind::AxisCount,
                    items.len() as u64,
                    "a physics node's output scales",
                )?;
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.number(item, "an output scale")?);
                }
                out
            }
            None => Vec::new(),
        };

        Ok(crate::puppet::PhysicsNode {
            length: self.number_or(fields, "length", 100.0)?,
            gravity: self.number_or(fields, "gravity", 1.0)?,
            frequency: self.number_or(fields, "frequency", 1.0)?,
            angle_damping: self.number_or(fields, "angle_damping", 0.5)?,
            length_damping: self.number_or(fields, "length_damping", 0.5)?,
            output,
            output_source: sources,
            output_scale: scales,
        })
    }

    fn transform(
        &mut self,
        owner: Id,
        path: &[&str],
        object: &[(String, Json)],
        key: &str,
    ) -> Result<Transform, Rejection> {
        let Some(value) = self.optional(object, key) else {
            return Ok(Transform {
                translation: [0.0, 0.0],
                rotation: 0.0,
                scale: [1.0, 1.0],
            });
        };
        let fields = self.object(value, key)?;
        self.extensions_preserved(owner, path, fields)?;
        self.sweep(owner, path, fields, TRANSFORM_KEYS);
        Ok(Transform {
            translation: match self.optional(fields, "translation") {
                Some(value) => self.pair(value, "a translation")?,
                None => [0.0, 0.0],
            },
            rotation: self.number_or(fields, "rotation", 0.0)?,
            scale: match self.optional(fields, "scale") {
                Some(value) => self.pair(value, "a scale")?,
                None => [1.0, 1.0],
            },
        })
    }

    /// The geometry a part or a mesh group carries.
    ///
    /// **Absent is an empty mesh**, not a malformed node. A mesh group
    /// deforms its children with its own mesh and a rig may carry one that
    /// deforms nothing yet, which is what `rigged:containment-permitted` and
    /// `rigged:near-limit` are made of; whether a *part* may be empty is a rule
    /// about the puppet rather than about the encoding, and is the validator's.
    fn mesh(&mut self, owner: Id, object: &[(String, Json)]) -> Result<Mesh, Rejection> {
        let Some(value) = self.optional(object, "mesh") else {
            return Ok(Mesh {
                vertices: Vec::new(),
                uvs: Vec::new(),
                triangles: Vec::new(),
            });
        };
        let fields = self.object(value, "a mesh")?;
        self.extensions_preserved(owner, &["mesh"], fields)?;
        self.sweep(owner, &["mesh"], fields, MESH_KEYS);

        let vertices = self.coordinates(fields, "vertices")?;
        crate::archive::within(LimitKind::VertexCount, vertices.len() as u64, "vertices")?;
        let uvs = self.coordinates(fields, "uvs")?;
        crate::archive::within(LimitKind::VertexCount, uvs.len() as u64, "uvs")?;

        let triangle_items = self.array(
            self.required(fields, "triangles", "a mesh")?,
            "a mesh's triangles",
        )?;
        crate::archive::within(
            LimitKind::TriangleCount,
            triangle_items.len() as u64,
            "triangles",
        )?;
        let mut triangles = Vec::with_capacity(triangle_items.len());
        for item in triangle_items {
            let corners = self.array(item, "a triangle")?;
            if corners.len() != 3 {
                return Err(malformed("a triangle does not name three corners"));
            }
            let mut out = [0u32; 3];
            for (slot, corner) in out.iter_mut().zip(corners) {
                *slot = corner
                    .as_u64()
                    .and_then(|index| u32::try_from(index).ok())
                    .ok_or_else(|| malformed("a triangle's corner is not a vertex index"))?;
            }
            triangles.push(out);
        }

        Ok(Mesh {
            vertices,
            uvs,
            triangles,
        })
    }

    fn coordinates(
        &self,
        object: &[(String, Json)],
        key: &str,
    ) -> Result<Vec<[f64; 2]>, Rejection> {
        let items = self.array(self.required(object, key, "a mesh")?, key)?;
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            out.push(self.pair(item, key)?);
        }
        Ok(out)
    }

    /// Refuses a tree nested past [`LimitKind::TreeDepth`].
    ///
    /// A chain that revisits a node is left alone rather than counted as
    /// infinitely deep: a cycle is [`Rule::TreeAcyclic`]'s to name, and
    /// reporting it as a depth limit would attribute `rigged:cycle` to the
    /// wrong rule.
    fn refuse_deep_trees(&self, nodes: &[Node]) -> Result<(), Rejection> {
        let limit = crate::archive::required_limit(LimitKind::TreeDepth)?;
        for node in nodes {
            let mut seen = vec![node.id];
            let mut depth: u64 = 1;
            let mut at = node.parent;
            while at != Id::NONE {
                if seen.contains(&at) {
                    break;
                }
                let Some(parent) = nodes.iter().find(|candidate| candidate.id == at) else {
                    break;
                };
                seen.push(at);
                depth += 1;
                if depth > limit {
                    return Err(Rejection::LimitExceeded {
                        limit: LimitKind::TreeDepth,
                        declared: limit,
                        detail: format!("the tree nests deeper than {limit}"),
                    });
                }
                at = parent.parent;
            }
        }
        Ok(())
    }

    // -------------------------------------------------- the parameters

    fn parameters(&mut self, object: &[(String, Json)]) -> Result<Vec<Parameter>, Rejection> {
        let Some(value) = self.optional(object, "parameters") else {
            return Ok(Vec::new());
        };
        let items = self.array(value, "parameters")?;
        crate::archive::within(LimitKind::ParameterCount, items.len() as u64, "parameters")?;

        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let fields = self.object(item, "a parameter")?;
            let id = self.identifier(
                self.required(fields, "id", "a parameter")?,
                "a parameter's id",
            )?;
            self.extensions_preserved(id, &[], fields)?;
            self.sweep(id, &[], fields, PARAMETER_KEYS);

            let name = self.text(
                self.required(fields, "name", "a parameter")?,
                "a parameter's name",
            )?;

            let axis_items = self.array(
                self.required(fields, "axes", "a parameter")?,
                "a parameter's axes",
            )?;
            crate::archive::within(
                LimitKind::AxisCount,
                axis_items.len() as u64,
                "a parameter's axes",
            )?;
            let mut axes = Vec::with_capacity(axis_items.len());
            for (n, axis) in axis_items.iter().enumerate() {
                axes.push(self.axis(id, n, axis)?);
            }

            let keypoint_items = self.array(
                self.required(fields, "keypoints", "a parameter")?,
                "a parameter's keypoints",
            )?;
            crate::archive::within(
                LimitKind::KeypointCount,
                keypoint_items.len() as u64,
                "keypoints",
            )?;
            let mut keypoints = Vec::with_capacity(keypoint_items.len());
            for keypoint in keypoint_items {
                keypoints.push(self.keypoint(keypoint)?);
            }

            out.push(Parameter {
                // Task 2 (acceptance surface): placeholders, as above.
                order: None,
                mappable: true,
                id,
                name,
                axes,
                keypoints,
            });
        }
        Ok(out)
    }

    fn axis(&mut self, owner: Id, index: usize, value: &Json) -> Result<Axis, Rejection> {
        let fields = self.object(value, "an axis")?;
        let step = index.to_string();
        let path = ["axes", step.as_str()];
        self.extensions_preserved(owner, &path, fields)?;
        self.sweep(owner, &path, fields, AXIS_KEYS);

        let range = self.pair(
            self.required(fields, "range", "an axis")?,
            "an axis's range",
        )?;

        // An absence is **not** the value *unknown*: *unknown* is a decision
        // the rigger declined to make and must be written (`VAL-17`,
        // `VAL-21`). Inferring it from an absence is the error this rule
        // exists to catch, so it is refused at every declared version.
        let polarity = match self.optional(fields, "polarity") {
            Some(value) => {
                let token = self.text(value, "an axis polarity")?;
                self.enumerated(
                    owner,
                    "polarity",
                    &token,
                    &[
                        ("bilateral", AxisPolarity::Bilateral),
                        ("unipolar", AxisPolarity::Unipolar),
                        ("unknown", AxisPolarity::Unknown),
                    ],
                    AxisPolarity::Unfamiliar,
                )
            }
            None => {
                self.violate(
                    Rule::AxisPolarityDeclared,
                    vec![owner],
                    Some("polarity"),
                    String::from("an axis declares no polarity, which is not the value unknown"),
                );
                AxisPolarity::Unknown
            }
        };
        Ok(Axis { range, polarity })
    }

    fn keypoint(&mut self, value: &Json) -> Result<Keypoint, Rejection> {
        let fields = self.object(value, "a keypoint")?;
        let id = self.identifier(
            self.required(fields, "id", "a keypoint")?,
            "a keypoint's id",
        )?;
        self.extensions_preserved(id, &[], fields)?;
        self.sweep(id, &[], fields, KEYPOINT_KEYS);

        let coordinates = self.array(
            self.required(fields, "position", "a keypoint")?,
            "a keypoint's position",
        )?;
        let mut position = Vec::with_capacity(coordinates.len());
        for coordinate in coordinates {
            position.push(self.number(coordinate, "a keypoint coordinate")?);
        }
        Ok(Keypoint {
            id,
            position,
            neutral: self.flag_or(fields, "neutral", false)?,
        })
    }

    // ---------------------------------------------------- the bindings

    fn bindings(&mut self, object: &[(String, Json)]) -> Result<Vec<Binding>, Rejection> {
        let Some(value) = self.optional(object, "bindings") else {
            return Ok(Vec::new());
        };
        let items = self.array(value, "bindings")?;
        crate::archive::within(LimitKind::BindingCount, items.len() as u64, "bindings")?;

        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let fields = self.object(item, "a binding")?;
            let id =
                self.identifier(self.required(fields, "id", "a binding")?, "a binding's id")?;
            self.extensions_preserved(id, &[], fields)?;
            self.sweep(id, &[], fields, BINDING_KEYS);

            let token = self.text(
                self.required(fields, "kind", "a binding")?,
                "a binding's kind",
            )?;
            // A binding's kind selects the shape of its values, so there is no
            // reading of an unfamiliar one that still yields values. It is
            // refused at every declared version, and that is a question for
            // `docs/format.md` recorded in `docs/wip/s2-format.md`.
            let kind = match token.as_str() {
                "transform" => BindingKind::Transform,
                "deformation" => BindingKind::Deformation,
                other => {
                    self.violate(
                        Rule::EnumeratedValueKnown,
                        vec![id],
                        Some("kind"),
                        format!("{other:?} is not a binding kind, so its values have no shape"),
                    );
                    continue;
                }
            };

            let values = self.binding_values(id, kind, fields)?;
            let stale = self.flag_or(fields, "stale", false)?;
            if stale {
                self.stale_bindings.push(id);
            }

            out.push(Binding {
                id,
                kind,
                parameter: self.identifier(
                    self.required(fields, "parameter", "a binding")?,
                    "a binding's parameter",
                )?,
                target: self.identifier(
                    self.required(fields, "target", "a binding")?,
                    "a binding's target",
                )?,
                values,
                stale,
            });
        }
        Ok(out)
    }

    fn binding_values(
        &mut self,
        owner: Id,
        kind: BindingKind,
        fields: &[(String, Json)],
    ) -> Result<BindingValues, Rejection> {
        let items = self.array(
            self.required(fields, "values", "a binding")?,
            "a binding's values",
        )?;
        crate::archive::within(
            LimitKind::KeypointCount,
            items.len() as u64,
            "a binding's values",
        )?;
        match kind {
            BindingKind::Transform => {
                let mut out = Vec::with_capacity(items.len());
                for (n, item) in items.iter().enumerate() {
                    let step = n.to_string();
                    let path = ["values", step.as_str()];
                    let transform = self.object(item, "a binding value")?;
                    self.extensions_preserved(owner, &path, transform)?;
                    self.sweep(owner, &path, transform, TRANSFORM_KEYS);
                    out.push(Transform {
                        translation: match self.optional(transform, "translation") {
                            Some(value) => self.pair(value, "a translation")?,
                            None => [0.0, 0.0],
                        },
                        rotation: self.number_or(transform, "rotation", 0.0)?,
                        scale: match self.optional(transform, "scale") {
                            Some(value) => self.pair(value, "a scale")?,
                            None => [1.0, 1.0],
                        },
                    });
                }
                Ok(BindingValues::Transform(out))
            }
            BindingKind::Deformation => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    let offsets = self.array(item, "a keypoint's offsets")?;
                    crate::archive::within(
                        LimitKind::VertexCount,
                        offsets.len() as u64,
                        "a keypoint's offsets",
                    )?;
                    let mut row = Vec::with_capacity(offsets.len());
                    for offset in offsets {
                        row.push(self.pair(offset, "a per-vertex offset")?);
                    }
                    out.push(row);
                }
                Ok(BindingValues::Deformation(out))
            }
        }
    }

    // -------------------------------------------------- the animations

    fn animations(&mut self, object: &[(String, Json)]) -> Result<Vec<Animation>, Rejection> {
        let Some(value) = self.optional(object, "animations") else {
            return Ok(Vec::new());
        };
        let items = self.array(value, "animations")?;
        crate::archive::within(LimitKind::AnimationCount, items.len() as u64, "animations")?;

        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let fields = self.object(item, "an animation")?;
            let id = self.identifier(
                self.required(fields, "id", "an animation")?,
                "an animation's id",
            )?;
            self.extensions_preserved(id, &[], fields)?;
            self.sweep(id, &[], fields, ANIMATION_KEYS);

            let name = self.text(
                self.required(fields, "name", "an animation")?,
                "an animation's name",
            )?;
            let track_items = self.array(
                self.required(fields, "tracks", "an animation")?,
                "an animation's tracks",
            )?;
            crate::archive::within(
                LimitKind::TracksPerAnimation,
                track_items.len() as u64,
                "tracks",
            )?;
            let mut tracks = Vec::with_capacity(track_items.len());
            for track in track_items {
                tracks.push(self.track(track)?);
            }
            out.push(Animation {
                id,
                name,
                tracks,
                // Task 2 (acceptance surface): placeholders, as above.
                end_behaviour: crate::puppet::EndBehaviour::Loop,
                triggers: Vec::new(),
            });
        }
        Ok(out)
    }

    fn track(&mut self, value: &Json) -> Result<Track, Rejection> {
        let fields = self.object(value, "a track")?;
        let id = self.identifier(self.required(fields, "id", "a track")?, "a track's id")?;
        self.extensions_preserved(id, &[], fields)?;
        self.sweep(id, &[], fields, TRACK_KEYS);

        let keyframe_items = self.array(
            self.required(fields, "keyframes", "a track")?,
            "a track's keyframes",
        )?;
        crate::archive::within(
            LimitKind::KeyframesPerTrack,
            keyframe_items.len() as u64,
            "keyframes",
        )?;
        let mut keyframes = Vec::with_capacity(keyframe_items.len());
        for (n, keyframe) in keyframe_items.iter().enumerate() {
            let frame = self.object(keyframe, "a keyframe")?;
            let step = n.to_string();
            let path = ["keyframes", step.as_str()];
            self.extensions_preserved(id, &path, frame)?;
            self.sweep(id, &path, frame, KEYFRAME_KEYS);

            let coordinates = self.array(
                self.required(frame, "value", "a keyframe")?,
                "a keyframe's value",
            )?;
            let mut coordinate_values = Vec::with_capacity(coordinates.len());
            for coordinate in coordinates {
                coordinate_values.push(self.number(coordinate, "a keyframe coordinate")?);
            }
            keyframes.push(Keyframe {
                // Task 2 (acceptance surface): the stored shape these carry is
                // task 3's. Placeholders only, chosen to keep the crate
                // compiling, and they are what makes the new acceptance tests
                // fail rather than not build.
                interpolation: crate::puppet::Interpolation::Linear,
                control_points: None,
                time: self.number(
                    self.required(frame, "time", "a keyframe")?,
                    "a keyframe's time",
                )?,
                value: coordinate_values,
            });
        }

        Ok(Track {
            id,
            parameter: self.identifier(
                self.required(fields, "parameter", "a track")?,
                "a track's parameter",
            )?,
            blend_duration: self.number_or(fields, "blend_duration", 0.0)?,
            keyframes,
        })
    }

    // ---------------------------------------------- the channel mappings

    fn channel_mappings(
        &mut self,
        object: &[(String, Json)],
    ) -> Result<Vec<ChannelMapping>, Rejection> {
        let Some(value) = self.optional(object, "channel_mappings") else {
            return Ok(Vec::new());
        };
        let items = self.array(value, "channel_mappings")?;
        crate::archive::within(
            LimitKind::ChannelMappingCount,
            items.len() as u64,
            "channel mappings",
        )?;

        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let fields = self.object(item, "a channel mapping")?;
            let id = self.identifier(
                self.required(fields, "id", "a channel mapping")?,
                "a channel mapping's id",
            )?;
            self.extensions_preserved(id, &[], fields)?;
            self.sweep(id, &[], fields, MAPPING_KEYS);

            let curve_fields = self.object(
                self.required(fields, "response_curve", "a channel mapping")?,
                "a response curve",
            )?;
            self.extensions_preserved(id, &["response_curve"], curve_fields)?;
            self.sweep(id, &["response_curve"], curve_fields, CURVE_KEYS);
            let point_items = self.array(
                self.required(curve_fields, "points", "a response curve")?,
                "a response curve's points",
            )?;
            crate::archive::within(
                LimitKind::ResponseCurvePoints,
                point_items.len() as u64,
                "response curve points",
            )?;
            let mut points = Vec::with_capacity(point_items.len());
            for point in point_items {
                points.push(self.pair(point, "a response curve point")?);
            }

            out.push(ChannelMapping {
                id,
                // A channel is named, not identified: it is supplied by a
                // source the puppet has never met ([I3]).
                channel: self.text(
                    self.required(fields, "channel", "a channel mapping")?,
                    "a channel name",
                )?,
                parameter: self.identifier(
                    self.required(fields, "parameter", "a channel mapping")?,
                    "a mapping's parameter",
                )?,
                source: match self.optional(fields, "source") {
                    Some(value) => self.text(value, "a source name")?,
                    None => String::new(),
                },
                response_curve: ResponseCurve { points },
            });
        }
        Ok(out)
    }

    // ---------------------------------------------------- the textures

    fn textures(&mut self, object: &[(String, Json)]) -> Result<Vec<TextureRecord>, Rejection> {
        let Some(value) = self.optional(object, "textures") else {
            return Ok(Vec::new());
        };
        let items = self.array(value, "textures")?;
        crate::archive::within(LimitKind::TextureCount, items.len() as u64, "textures")?;

        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let fields = self.object(item, "a texture")?;
            let id =
                self.identifier(self.required(fields, "id", "a texture")?, "a texture's id")?;
            self.extensions_preserved(id, &[], fields)?;
            self.sweep(id, &[], fields, TEXTURE_KEYS);

            let crop_items = self.array(
                self.required(fields, "crop", "a texture")?,
                "a texture's crop",
            )?;
            if crop_items.len() != 4 {
                return Err(malformed("a crop rectangle is not four numbers"));
            }
            let mut crop = [0u32; 4];
            for (slot, found) in crop.iter_mut().zip(crop_items) {
                let pixels = found
                    .as_u64()
                    .ok_or_else(|| malformed("a crop rectangle is not in whole pixels"))?;
                crate::archive::within(LimitKind::TextureDimension, pixels, "a crop rectangle")?;
                *slot = u32::try_from(pixels)
                    .map_err(|_| malformed("a crop rectangle does not fit a pixel count"))?;
            }

            out.push(TextureRecord {
                id,
                name: self.text(
                    self.required(fields, "name", "a texture")?,
                    "a texture's name",
                )?,
                authored_size: self.dimensions(
                    self.required(fields, "authored_size", "a texture")?,
                    "an authored size",
                )?,
                crop,
                // Filled from the archive once the record says which payload
                // belongs to it.
                payload: Vec::new(),
            });
        }
        Ok(out)
    }
}
