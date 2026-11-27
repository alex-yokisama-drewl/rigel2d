# Acceptance criteria — the editor's copy, paste and mirror operations

**Scope: this covers only the operations settled on 2026-11-26** — copy and paste components,
the rule for which parameters a paste shares and which it duplicates, mirroring, mirror axes,
cross-puppet paste, and the structured result every compound edit returns. **It does not cover
the rest of [authoring.md](authoring.md)**: meshing, the timeline, import and merge, undo and
first run have no criteria here, and a reader must not take this document for complete coverage
of that one. Those wait for the next cycle, as [plan.md](../wip/plan.md) records.

The criteria that decide whether what gets built satisfies those parts of
[authoring.md](authoring.md). Written from the requirements and [decisions.md](../decisions.md)
alone, in the position [puppet-acceptance.md](puppet-acceptance.md) states for itself.

## How to read this **[decided]**

**These are criteria, not tests.** Nothing is built, so none of it has been run and no criterion
here has been confirmed failing. Each is written to the point where the test that implements it
is obvious: a fixture, an operation on it, and an assertion.

**Every criterion asserts on the structured result of an operation and on the saved puppet,
never on an interface.** Neither application's interface is designed, and that is what makes
these writable now rather than after Studio exists — the position
[MSG-1](puppet-acceptance.md#g-what-the-user-is-told-decided) and
[ART-13](puppet-acceptance.md#m-adding-artwork-when-the-art-and-the-rig-disagree-proposed)
already take.

Each carries a `How`, with the tags [concerns/](../concerns/README.md) uses: `test`, `script`,
`judgment`, `gate`. Every `gate` names what the person looks at and what would count as wrong.
**Every criterion names what a wrong build would do that it catches**, which is what stops a
later reader deleting a row they cannot account for.

**A criterion is never firmer than the requirement it traces to.** Each section carries the
status tag of the section of [authoring.md](authoring.md) it is derived from. Identifiers are
stable, and a retired one keeps its identifier and is marked retired.

## The fixtures **[proposed]**

Fixtures are described by what they carry, never by how they are encoded. Each declares the
current format version unless its name says otherwise.

**Most of what these criteria run on is defined in
[puppet-acceptance.md](puppet-acceptance.md#the-fixtures-proposed)**, which is the one catalogue:
`rigged:paired` is the fixture for every mirroring and paste criterion here. One fixture
belongs to this document alone.

| Fixture | What it carries |
|---|---|
| `other-puppet` | A second, independent puppet whose identifiers deliberately collide with `rigged`'s — the same small integers naming different entities. The fixture for cross-puppet paste |

## A. Copy takes everything; paste chooses what lands **[decided]**

Traces to [Copy takes everything; paste chooses what
lands](authoring.md#copy-takes-everything-paste-chooses-what-lands-decided) and
[Which parameters a paste shares, and which it
duplicates](authoring.md#which-parameters-a-paste-shares-and-which-it-duplicates-decided).
All of these run on `rigged:paired`.

| ID | Operation and assertion | How |
|---|---|---|
| **PASTE-1** | **Copy is one command over a whole node**, and whole-node paste lands the subtree, its meshes, its bindings and its deformations. Assert the pasted subtree's shape, mesh data, bindings and deformation values equal the source's, with fresh identifiers throughout and no identifier shared with the source. **Wrong build:** drops bindings or deformations, which is the single most-cited friction this feature exists to remove, re-created | `test` |
| **PASTE-2** | **Mesh-only paste does not carry the texture reference.** Paste the mesh of a part referencing texture A onto a part referencing texture B. Assert the target still references B, that A gains no new reference, and that the mesh is scaled into B's coordinate space by the same rule [ART-3](puppet-acceptance.md#m-adding-artwork-when-the-art-and-the-rig-disagree-proposed) asserts for replacement art of different dimensions, with the evaluated geometry matching that rule's expectation. **Wrong build:** carries the texture, which turns a heterochromia mesh paste into a whole-node paste and replaces the rigger's second drawing without saying so | `test` |
| **PASTE-3** | The mesh is **scaled, not copied raw**, when the two textures differ in dimensions. Assert against the hand-derived expectation of the defined scaling, not against the implementation. **Wrong build:** copies coordinates, so the mesh lands off its artwork and reads as a rigging mistake rather than a bug | `test` |
| **PASTE-4** | **Pasting rigging creates bindings, never parameters.** Paste rigging whose bindings name a parameter also bound to a node outside the pasted set. Assert the parameter count is unchanged; that the parameter's stored bytes — range, polarity, keypoint positions, neutral keypoint — are identical; and that the node outside evaluates exactly as before at every keypoint. **Wrong build:** rewrites or duplicates the shared parameter, which silently re-means every other binding on it and every channel mapping Live wrote into it | `test` |
| **PASTE-5** | Each paste is **one undo step**. For each form, plain and mirrored, assert one undo returns the serialisation to the pre-paste bytes. **Wrong build:** undoes a compound operation in pieces, leaving a half-pasted rig that no operation produced and the validator may reject | `test` |
| **PASTE-6** | Each form is available **plain or mirrored** — the whole matrix, asserted over the enumerable operation set. **Wrong build:** offers mirroring for whole nodes only, so mirroring rigging — the case the requirement's own examples are about — stays manual | `script` |
| **PASTE-7** | **The share-or-duplicate choice is per parameter, by reach.** Mirror-paste the group in `rigged:paired`. Assert the parameter used only inside is **duplicated** — a new parameter with a new identifier, the pasted bindings naming the duplicate, the original's stored values unchanged, and the two sides thereafter moving independently — and that the parameter also used outside is **shared**: no new parameter, the pasted bindings naming the original. **Wrong build:** applies either rule wholesale, giving a mirrored eye that can never wink or a mirrored earring that stops following the head | `test` |
| **PASTE-8** | Reach is computed over **nodes**, and over the pasted set as it lands. Assert a parameter bound only by nodes inside the set is duplicated even when a channel mapping or physics also names it, and that one bound by any node outside is shared even when only one such node exists. **Wrong build:** counts drivers as reach, so a parameter Live mapped is shared and the two sides are welded together by a mapping neither rigger can see | `test` |
| **PASTE-9** | The rigger can **override** the decision per parameter, and the result reports what happened either way. Assert an overridden parameter takes the other treatment and that the result still names it as shared or duplicated. **Wrong build:** a rule with no escape, or an override accepted and silently ignored | `test` |
| **PASTE-10** | **A duplicated parameter carries its physics, and neither channel mappings nor animations.** Assert the duplicate is driven by a **new physics entity** with its own identifier driving only the duplicate; that the original's physics is unchanged and still drives only the original; that the duplicate has **no channel mapping and no animation**, while the original's animation still drives the original and gains no reference to the duplicate; and that the result **names every parameter in that state**. **Wrong build:** copies the mapping, so one channel feeds both sides and the duplication bought nothing; copies the animation, so a timeline written about one eye now moves both and the wink it was duplicated for is impossible; points both parameters at one physics, so the two sides can never differ; or says nothing, so the puppet moves in Studio and is dead on stream | `test` |
| **PASTE-11** | A paste never produces a parameter with **two drivers** ([decisions.md](../decisions.md)). Assert the result of every paste in this group validates, specifically against the one-driver rule. **Wrong build:** gives a duplicate both copied physics and a copied mapping, which the validator catches at save — after the rigger has done an hour's work on top of it | `test` |
| **PASTE-12** | A person mirror-pastes the rigging of a finished eye onto the other and reads the result. **Wrong looks like:** the duplicated parameters' *fed by nothing* state not visible without opening another panel; parameters listed by identifier rather than by name; a result that reads as unqualified success with no mention that something now needs a channel mapping before it will move on stream | `gate` |

## B. Mirroring reads a parameter and never writes one **[decided]**

Traces to [Mirroring reads a parameter and never writes
one](authoring.md#mirroring-reads-a-parameter-and-never-writes-one-decided), to
[Mirror axes](authoring.md#mirror-axes-decided) and to [A mirror rewrites
bindings and never writes a parameter](../decisions.md).

| ID | Operation and assertion | How |
|---|---|---|
| **MIR-1** | **Mirroring a binding rewrites that binding and nothing else.** Mirror the bindings on a node bound to a parameter that also drives a node outside. Assert the parameter's stored bytes are identical — polarity, range, keypoint positions, neutral keypoint — that the other node's evaluated geometry is unchanged at every keypoint, and that the mirrored binding's values are the reflection. **Wrong build:** reverses the parameter's axis, which re-means every other binding on it and every mapping Live wrote into it. Three readers of this design reached that wrong conclusion independently, which is why it is a criterion rather than a note | `test` |
| **MIR-2** | **Axis polarity decides whether reversal applies**, asserted over both kinds in one run: on a bilateral axis the mirrored binding's keypoints hold each other's values; on a unipolar axis they do not. **Wrong build:** reverses everything, so *eye open* opens the eye when it should shut; or reverses nothing, so head turn mirrors to a rig that leans the same way on both sides | `test` |
| **MIR-3** | **An axis declaring *unknown* reverses nothing, and the result says so.** Mirror the group in `rigged:paired`, one of whose parameters declares *unknown* on an axis — a current-version file, because *unknown* is a value a puppet may legitimately carry. Assert the bindings on that axis are reflected geometrically but **not** reversed, that the result names each parameter left alone so the rigger can set its polarity and mirror again, and that nothing about the parameter is written, the value included. **Wrong build:** guesses bilateral, which is wrong for every head-turn parameter the import brought in and leaves no trace that a guess was made; or writes the axis to bilateral on the rigger's behalf, which is the one thing a mirror may never do to a parameter | `test` |
| **MIR-4** | **Mirroring twice returns the original, byte for byte** — for a mesh, for a binding, and for a subtree containing both, across both mirror axes, and **including each reflected part's texture flip**, which toggles twice and so returns to where it started. **Wrong build:** accumulates rounding, or recomputes the axis from the current content rather than using the axis, so the two mirrors do not cancel; or toggles the flip on only one of the two passes, so the artwork ends reversed in a file that is otherwise identical. This is the first thing a rigger tries, and the drift is invisible in a single pass | `test` |
| **MIR-5** | A mirror produces **independent data and no link**. Mirror a subtree; edit one side. Assert the other is unchanged in bytes and in evaluated geometry, and that the saved file carries no reference from either to the other. **Wrong build:** stores a link, which is puppet data with no identity, no validation rule and no defined behaviour for the case riggers hit routinely — deliberately making the two sides differ | `test` |
| **MIR-6** | **A mirror preserves vertex index correspondence**: the reflected mesh holds the same vertices in the same order, so deformations transfer onto it **by position in the list**. Mirror a mesh, and mirror-paste one onto a part carrying deformations. Assert the vertex list is the same length in the same order, that the vertex at each index is the reflection of the vertex that was at that index, and that each deformation offset lands on the index it was authored against — against a hand-derived expectation rather than the implementation's own output. **Wrong build:** renumbers, reorders or re-winds the vertices, which scrambles every offset carried across: the part still renders, nothing is flagged, and the damage appears only when the parameter moves. Note this is **identity on indices**, not a reflected permutation of them — a build that carefully maps each index to its mirrored counterpart is also wrong, and it is the plausible wrong answer | `test` |
| **MIR-7** | Every mirrored operation's **result names the axis it used**, derived or stored. Assert it for each scope and for both axes, and that the named axis is the one the geometry was reflected about. **Wrong build:** reflects about the derived axis while the puppet carries a moved one, which is correct-looking at the puppet's centre and wrong everywhere else | `test` |
| **MIR-8** | Mirroring is **one undo step** per scope, restoring the pre-mirror bytes. **Wrong build:** as PASTE-5 | `test` |
| **MIR-10** | **A mirror toggles the texture flip on the axis it mirrored**, for every part it reflects. Mirror across the left–right axis from each of the four flip states, in place and as a mirrored paste. Assert each reflected part's left–right flip is toggled and its top–bottom flip untouched, and the converse for the top–bottom axis; assert every part reflected is covered, not only the one the rigger selected. **Wrong build:** reflects the mesh and leaves the sampling, so the highlight sits on the wrong side of the second eye — the case the whole sharing arrangement exists to serve, and one that reads as an art mistake rather than a tool bug | `test` |
| **MIR-11** | The toggle **creates nothing**. Assert the texture count, the payload set and every other part's texture reference are unchanged by a mirror, and that the reflected part still references the same texture entity as the part it was mirrored from. **Wrong build:** materialises a reversed copy of the artwork to achieve the reversal, so every mirrored pair doubles its texture memory — [FLIP-3](puppet-acceptance.md#texture-flip-axis-polarity-and-the-mirror-axes-proposed)'s failure arriving by the mirror rather than by the flip | `test` |
| **MIR-9** | A person mirrors a rigged eye and then moves each parameter through its range. **Wrong looks like:** the second eye's artwork reversed the wrong way — a highlight or a lash on the inside where it should be on the outside, which [MIR-10](#b-mirroring-reads-a-parameter-and-never-writes-one-decided) asserts structurally and a person sees instantly; the un-mirrored eye moving when only one side was mirrored; the mirrored eye moving the wrong way at the ends of a bilateral axis; a unipolar expression inverted; the result naming no parameter when an axis declared *unknown* | `gate` |

## C. Pasting into a different puppet **[decided]**

Traces to [Pasting into a different
puppet](authoring.md#pasting-into-a-different-puppet-decided).

| ID | Operation and assertion | How |
|---|---|---|
| **XPASTE-1** | A paste into another puppet **duplicates, always, and says so**. Paste from `rigged` into `other-puppet`. Assert no pasted binding names a pre-existing destination parameter, that every one names a parameter the paste created, and that the result states the mode and that sharing has no meaning across files. **Wrong build:** offers a share mode it cannot honour, which produces a binding naming a parameter in a file the puppet will never see again | `test` |
| **XPASTE-2** | **Every identifier is reissued from the destination**, asserted on deliberately colliding identifier spaces. Assert every pasted entity carries a destination-issued identifier distinct from every pre-existing one, that no pasted reference resolves to a pre-existing destination entity, and that each resolves to its own pasted counterpart. **Wrong build:** keeps the source's numbers, giving a part that draws another part's texture — a wrong reference in a file that never looks malformed, because nothing dangles for the validator to catch | `test` |
| **XPASTE-3** | Reissued identifiers are also **not drawn from the destination's retired ones**, whether retired by deletion or by collection. **Wrong build:** [ID-4](puppet-acceptance.md#a-every-entity-has-a-stable-identity-proposed)'s failure arriving by a route ID-4 does not exercise | `test` |
| **XPASTE-4** | A reference from the pasted set to something **not copied** follows its declared kind: constitutive, and the paste is **refused naming what is missing**; incidental, and the reference **drops** and the paste proceeds. Table-driven from the same declaration [ART-15](puppet-acceptance.md#m-adding-artwork-when-the-art-and-the-rig-disagree-proposed) checks. **Wrong build:** drops a constitutive reference, producing a part with no texture — a malformed puppet Studio made itself; or refuses on an incidental one, blocking a paste that was fine and leaving the rigger no way to move a masked part between files | `test` |
| **XPASTE-5** | A cross-puppet paste is **one undo step** and leaves the **source puppet** unchanged in memory and in bytes. **Wrong build:** renumbers the source while reissuing, which damages the file the rigger was copying from and which they may not save for hours | `test` |
| **XPASTE-6** | A **mirrored paste uses the destination's mirror axes.** Move `other-puppet`'s left–right axis away from its centre and leave `rigged`'s derived; mirror-paste between them. Assert the reflection is about the destination's axis, that the result names that axis ([MIR-7](#b-mirroring-reads-a-parameter-and-never-writes-one-decided)), and that the source's axes are neither read nor written. **Wrong build:** carries the source's axis, so a fragment lands correctly in the puppet it came from and off-centre in every other — and the error scales with how far apart the two puppets' centres are | `test` |
| **XPASTE-7** | **Artwork arriving by paste deduplicates against the destination** by content hash, exactly as an import does. Paste a part into a puppet that already holds the same drawing. Assert one texture entity results, that both parts reference it, and that the saved file holds one payload — [TEX-7](puppet-acceptance.md#b-a-texture-is-not-a-part-decided)'s assertion, reached by paste. Assert too that artwork matching a payload the destination has **collected** yields a new entity, not a match ([COLL-7](puppet-acceptance.md#c-a-textures-life-is-bound-to-the-parts-that-reference-it-decided)). **Wrong build:** deduplicates on import only, which makes paste the cheapest route to duplicating artwork and the one the rigger reaches for most | `test` |

## D. Every compound edit reports what it did **[decided]**

Traces to [Every compound edit reports what it
did](authoring.md#every-compound-edit-reports-what-it-did-decided). Asserted on the structured
result, which is what makes this section checkable before either interface exists.

| ID | Operation and assertion | How |
|---|---|---|
| **RES-1** | Paste, mirror, import and the collection of artwork nothing draws each return a structured result naming what was **created, shared, flagged, collected and left unchanged**. Assert each category is present in each operation's result, **empty where nothing applies rather than absent**. **Wrong build:** omits a category, which is ambiguous between *nothing happened* and *not implemented* — and the reader who needs it most is the one who cannot tell the difference | `test` |
| **RES-2** | **An operation that changes something the rigger did not point at names it.** Table-driven over the cases the requirement names: a mirror leaving an unknown-polarity parameter alone; a paste sharing a parameter; a re-import splitting a shared texture; a mesh paste leaving a deformation stale; an edit collecting a texture. For each, assert the entity appears in the result under the right category and is identifiable by name. **Wrong build:** each of these is correct behaviour and each is invisible, so a build that reports none of them passes every other criterion and the beginner finds out on stream | `test` |
| **RES-3** | Every compound operation **has** a structured result. Assert over the enumerable operation set that each compound operation declares one; an operation without one fails the check. **Wrong build:** adds a compound operation with no result, which RES-1 and RES-2 would pass by never reaching it | `script` |
| **RES-4** | A person performs a mirrored rigging paste and a re-import split and reads both results. **Wrong looks like:** entities listed by identifier rather than by name; *left unchanged* so long that what changed is buried in it; a collected texture and a shared parameter presented as the same kind of event; nothing distinguishing what the rigger asked for from what followed | `gate` |

## Criteria not written **[open]**

Each is a place where [authoring.md](authoring.md) does not say enough for a criterion to be
written without inventing the answer, or where the criterion belongs to an interface that does
not exist. Recorded rather than guessed.

| What could not be written | What it needs |
|---|---|
| That mirroring is **convenient** with mouse and keyboard, and that a mirror axis can be moved precisely enough to be useful | Interaction quality, against an interface that does not exist. It belongs to Studio's interface gate when there is one, and it is named here so that it is not lost: the mirror axis is the first piece of authoring-only data a rigger must position by hand |
| The **form** in which a result names a derived mirror axis | [MIR-7](#b-mirroring-reads-a-parameter-and-never-writes-one-decided) asserts the axis is named and that it is the one used. What a derived axis is called in the result — a coordinate, a word, the puppet's centre — is an interface question with no requirement behind it yet |
| **Whether a duplicate that carried physics counts as *driven by nothing*.** *A duplicated parameter carries its physics, and neither channel mappings nor animations* — and then *the duplicate arrives driven by nothing, and the result names every parameter in that state*. A duplicate whose original was physics-driven has a driver, so it is not in that state; one whose original was channel-fed is. [PASTE-10](#a-copy-takes-everything-paste-chooses-what-lands-decided) is written to that reading, which makes the reported set the mapping-fed duplicates only. The collision is with [One parameter has one driver](../decisions.md), where physics **is** a driver | One word. Either *driven* becomes *fed*, restoring the earlier sentence's narrower sense, or the sentence says that a physics-carrying duplicate is reported too. The second changes what the result must list, so the criterion cannot assert both |
