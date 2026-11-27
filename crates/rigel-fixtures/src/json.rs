//! Canonical JSON, and the one escape hatch the corpus needs.
//!
//! `docs/format.md` asks for "keys sorted, no insignificant whitespace, a fixed
//! number format, UTF-8". `serde_json`'s map is a `BTreeMap` unless the
//! `preserve_order` feature is on, and `to_string` emits no whitespace, so the
//! first two come for free and the third is serde_json's own shortest
//! round-tripping float format.
//!
//! **The escape hatch.** Several fixtures have to carry bytes that no in-memory
//! JSON value can hold — `NaN`, a number that overflows to infinity, or an
//! object whose keys are deliberately *not* sorted. A [`Raws`] registry hands
//! out an opaque placeholder string, and [`Doc::to_json`] substitutes the
//! literal text back in after serialisation. The placeholder is plain ASCII, so
//! it is never escaped on the way through.

use serde_json::{Map, Value};

/// Literal JSON text held aside until after serialisation.
#[derive(Clone, Default)]
pub struct Raws(Vec<String>);

impl Raws {
    /// Register literal text and return the placeholder that stands for it.
    pub fn add(&mut self, text: impl Into<String>) -> Value {
        let index = self.0.len();
        self.0.push(text.into());
        Value::String(format!("@@RAW{index}@@"))
    }

    fn apply(&self, mut text: String) -> String {
        for (index, literal) in self.0.iter().enumerate() {
            text = text.replace(&format!("\"@@RAW{index}@@\""), literal);
        }
        text
    }
}

/// A JSON document under construction, with its registry of literal text.
#[derive(Clone)]
pub struct Doc {
    pub value: Value,
    pub raws: Raws,
}

impl Doc {
    pub fn new(value: Value) -> Self {
        Doc {
            value,
            raws: Raws::default(),
        }
    }

    pub fn to_json(&self) -> String {
        self.raws
            .apply(serde_json::to_string(&self.value).expect("fixture JSON is serialisable"))
    }

    /// The top-level object.
    pub fn obj(&mut self) -> &mut Map<String, Value> {
        self.value.as_object_mut().expect("document is an object")
    }

    /// One member of a top-level array, found by its `id`.
    ///
    /// Panics rather than returning an option: a fixture naming an entity that
    /// is not there is a mistake in the builder, and it should stop the build
    /// rather than silently produce a fixture with no defect in it.
    pub fn entity(&mut self, collection: &str, id: u64) -> &mut Map<String, Value> {
        let items = self
            .obj()
            .get_mut(collection)
            .and_then(Value::as_array_mut)
            .unwrap_or_else(|| panic!("no `{collection}` collection"));
        items
            .iter_mut()
            .find(|item| item.get("id").and_then(Value::as_u64) == Some(id))
            .unwrap_or_else(|| panic!("no entity {id} in `{collection}`"))
            .as_object_mut()
            .expect("entity is an object")
    }

    pub fn node(&mut self, id: u64) -> &mut Map<String, Value> {
        self.entity("nodes", id)
    }

    pub fn parameter(&mut self, id: u64) -> &mut Map<String, Value> {
        self.entity("parameters", id)
    }

    pub fn binding(&mut self, id: u64) -> &mut Map<String, Value> {
        self.entity("bindings", id)
    }

    pub fn texture(&mut self, id: u64) -> &mut Map<String, Value> {
        self.entity("textures", id)
    }

    /// One track of one animation, found by the pair of identifiers.
    pub fn track(&mut self, animation: u64, track: u64) -> &mut Map<String, Value> {
        let tracks = self
            .entity("animations", animation)
            .get_mut("tracks")
            .and_then(Value::as_array_mut)
            .expect("animation has tracks");
        tracks
            .iter_mut()
            .find(|item| item.get("id").and_then(Value::as_u64) == Some(track))
            .unwrap_or_else(|| panic!("no track {track}"))
            .as_object_mut()
            .expect("track is an object")
    }

    /// One keypoint of one parameter.
    pub fn keypoint(&mut self, parameter: u64, keypoint: u64) -> &mut Map<String, Value> {
        let keypoints = self
            .parameter(parameter)
            .get_mut("keypoints")
            .and_then(Value::as_array_mut)
            .expect("parameter has keypoints");
        keypoints
            .iter_mut()
            .find(|item| item.get("id").and_then(Value::as_u64) == Some(keypoint))
            .unwrap_or_else(|| panic!("no keypoint {keypoint}"))
            .as_object_mut()
            .expect("keypoint is an object")
    }

    /// Add a node, keeping `nodes` in depth-first preorder.
    ///
    /// Preorder is the canonical order of the collection, and in it a node's
    /// descendants are the contiguous run that follows it. So the insertion
    /// point is the end of the parent's run, which is also what makes the new
    /// node the parent's last child — its place in draw order.
    pub fn push_node(&mut self, node: Value) {
        let parent = node
            .get("parent")
            .and_then(Value::as_u64)
            .expect("a node declares its parent");
        let nodes = self
            .obj()
            .get_mut("nodes")
            .and_then(Value::as_array_mut)
            .expect("nodes is an array");

        let field = |node: &Value, key: &str| node.get(key).and_then(Value::as_u64).unwrap_or(0);
        let ids: Vec<u64> = nodes.iter().map(|n| field(n, "id")).collect();
        let parents: Vec<u64> = nodes.iter().map(|n| field(n, "parent")).collect();

        let Some(start) = ids.iter().position(|id| *id == parent) else {
            nodes.push(node);
            return;
        };
        let mut subtree = vec![parent];
        let mut end = start + 1;
        while end < ids.len() && subtree.contains(&parents[end]) {
            subtree.push(ids[end]);
            end += 1;
        }
        nodes.insert(end, node);
    }
}

/// Convenience: an entity's `x` block, created if it is not there.
pub fn extension_block(entity: &mut Map<String, Value>) -> &mut Map<String, Value> {
    entity
        .entry("x")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .expect("x is an object")
}
