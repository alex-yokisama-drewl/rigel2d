# What simulation constants should one physics node store?

**Researched:** 2026-11-27 · **Feeds:** the physics-node field set in the puppet format (stored names, meanings, ranges) · **Verdict:** Option D, built on inochi2d's physical parameterisation — one spring-pendulum model, six stored constants, no model-type switch — not Cubism's dimensionless per-vertex factors and not a bare four-knob pendulum.

## Verdict

**Take inochi2d's parameterisation, pruned (A, reduced to one model), and reject Cubism's (B).** The decisive evidence is that inochi2d's constants are physical quantities that map one-to-one onto a single node (a length in puppet pixels, a frequency in Hz, damping ratios, a gravity multiplier), while Cubism's are dimensionless fudge factors that only make sense inside a chain, tied to a hard-coded 30 fps reference.

Proposed stored field set, per physics node. Ranges are **my proposal**: neither incumbent declares any (see Evidence), so they are engineering limits to be confirmed by the runtime's stability test, not inherited values.

| Field | Meaning | Unit | Default | Proposed range | Origin |
|---|---|---|---|---|---|
| `length` | Rest distance from the node's anchor to the simulated bob | puppet pixels | 100 | finite, > 0 | inochi2d `length` |
| `gravity` | Multiplier on the puppet-wide gravity; 0 turns gravity off | ratio | 1.0 | finite, >= 0 | inochi2d `gravity` |
| `frequency` | Resonant frequency of the restoring spring; the "stiffness" knob, and what option C calls restoring stiffness | Hz | 1.0 | finite, > 0, upper bound set by integrator stability at the fixed step | inochi2d `frequency` |
| `angle_damping` | Damping of tangential (swinging) motion | damping ratio | 0.5 | finite, >= 0 | inochi2d `angle_damping` |
| `length_damping` | Damping of radial (stretch) motion | damping ratio | 0.5 | finite, >= 0 | inochi2d `length_damping` |
| `output_scale` | Gain from simulated offset to the driven parameter | ratio | 1.0 (per axis if the bound parameter is two-dimensional) | finite | inochi2d `output_scale`; Cubism output `Scale` |

Dropped from inochi2d's stored set, with reasons:

- **`model_type`** (pendulum vs spring pendulum). The rigid pendulum ignores `frequency` and `length_damping`, so it is a strict subset of the spring model. A switch that silently disables two other knobs is a beginner trap (question 5). Store only the spring pendulum; a very high `frequency` approximates the rigid case.
- **`local_only`.** In our format the node sits in a tree and displaces its own transform; whether the anchor is world or local is a consequence of that design, not a per-node choice a rigger should make. Re-check when the evaluation design settles the anchor.
- **`map_mode`** (angle-length, xy, yx, length-angle). That is how the simulated bob is mapped onto the one driven parameter. It belongs to the parameter-binding part of the node, not to the simulation constants. It is flagged here because the incumbent stores it beside the constants.

Dropped from Cubism, with reasons: per-vertex `Mobility`, `Delay`, `Acceleration`, `Radius`; input `Weight`, `Reflect`, `Type`; `Normalization` position/angle ranges; per-output `VertexIndex`, `Weight`. These exist because Cubism's pendulum is driven by *parameters* (head angle, body angle) that must be normalised, weighted and flipped, and because one setting holds a whole chain. Our node is driven by its own parent's motion through the tree and chains by nesting, so none of that has a referent.

**Not settled by the evidence:** the numeric ranges above, the exact damping-ratio semantics (I could not read the inochi2d equations of motion, only that "ratio" is the doc term), and whether riggers want a second axis (below). Each is listed under *What would change this*.

## Evidence

### 1. The field sets

**inochi2d `SimplePhysics`.** Source: [`source/inochi2d/nodes/legacy/simplephysics.d`](https://github.com/Inochi2D/inochi2d/blob/main/source/inochi2d/nodes/legacy/simplephysics.d) on `main`, read 2026-11-27. The path says `legacy`, so the node may be reworked in the 0.9 line (see *What would change this*). Serialized keys: `target`, `model_type`, `map_mode`, `gravity`, `length`, `frequency`, `angle_damping`, `length_damping`, `output_scale`, `local_only`.

| Field | Default | Unit / meaning per source |
|---|---|---|
| `gravity` | 1.0 | Scale on puppet gravity (1.0 is puppet gravity); multiplied by the puppet's own `physicsGravity` and pixels-per-metre |
| `length` | 100 | Rest length in pixels |
| `frequency` | 1 | Resonant frequency, Hz. Used by the spring model only |
| `angle_damping` | 0.5 | Angular damping ratio. Used by both models |
| `length_damping` | 0.5 | Length damping ratio. Spring model only |
| `output_scale` | (1, 1) | Two-component gain on mapped output |
| `model_type` | pendulum | `pendulum` or `spring_pendulum` |
| `map_mode` | angle_length | `angle_length`, `xy`, `yx`, `length_angle` |
| `local_only` | false | Anchor from local transform rather than world matrix |

No min or max is declared for any property; defaults only. The only numeric guards are a timestep cap of 10 seconds, fixed 0.01 s sub-steps, and a finiteness check in the integrator that reverts a step producing a non-finite value ([`core/phys/system.d`](https://github.com/Inochi2D/inochi2d/blob/main/source/inochi2d/core/phys/system.d), RK4). Source also defines the length-damping property twice, which looks like a copy error. Each node is one anchor and one bob, so inochi2d is **already per-node**: chains are made by stacking nodes. This matches our shape directly.

**Cubism `.physics3.json`.** Source: the Cubism framework parser, [`cubismphysicsjson.ts`](https://cdn.jsdelivr.net/npm/live2d-vuenext@1.1.10/Framework/src/physics/cubismphysicsjson.ts) (third-party mirror of the official framework; I could not reach Live2D's own format page, so the key list is from parser code, not a spec), and the update code in [`cubismphysics.ts`](https://cdn.jsdelivr.net/npm/live2d-renderer@0.6.6/framework/src/physics/cubismphysics.ts). Read 2026-11-27.

- `Meta.EffectiveForces`: `Gravity` and `Wind`, each `X`, `Y`. **Global, not per setting**, and the SDK manual says runtime options overwrite them (default gravity (0, -1), wind (0, 0)) per [docs.live2d.com SDK physics](https://docs.live2d.com/en/cubism-sdk-manual/physics).
- Per `PhysicsSettings` entry: `Normalization.Position` and `.Angle` (`Minimum`, `Default`, `Maximum`); `Input[]` (`Source.Id`, `Weight`, `Reflect`, `Type`); `Output[]` (`Destination.Id`, `VertexIndex`, `Scale`, `Weight`, `Reflect`, `Type`); `Vertices[]` (`Position X/Y`, `Mobility`, `Delay`, `Acceleration`, `Radius`).
- What the four per-vertex fields do, from the update code: `Radius` is the fixed distance of a particle from its parent. `Mobility` multiplies the retained velocity each step (damping-like, dimensionless). `Delay` is scaled by `dt * 30` and multiplies both velocity carry-over and, squared, the force term (an inertia/time-scale factor; the 30 is a hard-coded 30 fps reference). `Acceleration` scales the gravity direction before wind is added. Input weights are divided by 100.
- Units and ranges: none declared. Observed in one sample file ([haru.physics3.json mirror](https://cdn.jsdelivr.net/npm/@white-matrix/live2d-lib@0.0.1/resources/haru/haru.physics3.json)): Mobility 0.95–1, Delay 0.8–1, Acceleration 1–1.5, Radius 0–15, input normalisation -10 to 10, output Type Angle with Scale 2. That is one shipped file, an example of values rather than a range. The editor manual suggests 0.7–0.99 as a rule of thumb for "ease of swinging" ([Cubism editor, Physics Settings](https://docs.live2d.com/en/cubism-editor-manual/physics-operation/)).
- Editor-side names for the pendulum settings: Duration, Ease of swinging, Reaction time, Speed of convergence ([editor manual](https://docs.live2d.com/en/cubism-editor-manual/physical-operation-setting/)). The page does not say which file field each is; the pairing with `Radius`, `Mobility`, `Delay`, `Acceleration` in that order is **my inference** from the value conventions (1 is neutral for reaction time and convergence; 0.7–0.99 for ease of swinging matches `Mobility`).
- Calculation rate is a group-level setting, 15/30/60/120 or custom, default 60 ([editor manual](https://docs.live2d.com/en/cubism-editor-manual/physics-operation/)). A rate in the file is a runtime concern for us, not a node constant.

### 2. Do the two map onto each other?

Mostly not, and the places they agree are thin.

| Concept | inochi2d | Cubism | Agree? |
|---|---|---|---|
| Rest length | `length`, pixels | `Radius`, unitless in file | Yes, same idea; different units. Cubism's is relative to the chain's output scale |
| Gravity | `gravity` multiplier, per node | global `Meta` gravity vector, plus per-vertex `Acceleration` scaling it | Partly: both scale a puppet-wide gravity per element |
| Damping | `angle_damping`, `length_damping` as ratios | `Mobility` (velocity retention) | Same role, different form: ratio vs retention factor. Not convertible by rename |
| Restoring spring | `frequency` in Hz | none explicit; `Delay` and `Acceleration` jointly set response speed | No. Cubism has no named stiffness |
| Output gain | `output_scale` | output `Scale` | Yes |
| Drive input | node motion through the tree | `Input[]` of named parameters, normalised, weighted | No, structurally different |
| Chain | stack nodes | one setting with an ordered `Vertices[]` | No; this is the per-chain vs per-node difference the brief already settled |

Only length and output gain are a shared vocabulary. Damping is shared as a role but not as a number. **There is no settled common vocabulary that would justify adopting Cubism names or its factor semantics.**

### 3. One dimension or two

- inochi2d: the bob has a 2D position. The rigid model integrates an angle; the spring model integrates 2D position and velocity. Output maps to a one-dimensional parameter (x component only) or two-dimensional parameter (each component scaled), selected by `map_mode`. So the *simulation* is planar and the *output* is 1D or 2D by the bound parameter's kind.
- Cubism: output `Type` is Angle (the sample file uses Angle for both settings) or translation X/Y per output, from the same pendulum, via `getOutputTranslationX/Y` and `getOutputAngle` in the framework code. The pendulum itself is planar.
- "Two axes" for a 2D drawing is the image plane, which both already simulate. Whether riggers want independent per-axis constants (different damping on x and y) is **not supported by any source I found**. The only related community signal is a bug report that gravity seemed to act only horizontally on the spring ([inochi-creator #479](https://github.com/Inochi2D/inochi-creator/issues/479), 2025-02-02), which is a correctness complaint, not a request for a second axis. Adjudicating that report is out of scope; the runtime's tests should cover it.

### 4. Which knobs riggers tune

Weak evidence, and I did not find rigger-written guides for either tool's constants in this pass.

- Live2D's own editor manual steers riggers to the output side: an **Output Maximum (%)** readout with an auto "Increase Output" button that sets `Scale` so the swing reaches 100% of the parameter range. A scale near 20 is suggested for ten-stage skinning ([editor manual](https://docs.live2d.com/en/cubism-editor-manual/physical-operation-setting/)). That indicates output scale is the constant that most needs per-rig adjustment, and that the tool automates it.
- Live2D's pendulum constants sit near 1 in the sample file, consistent with them being left near preset values, but this is one sample, not a corpus.
- inochi2d #479 (2025-02-01, open, milestone 0.9 as of 2025-06-09) shows riggers working around the limits by stacking several simple-physics nodes on one parameter with heavy damping and no frequency or gravity. That is evidence that `angle_damping` and `frequency` are the knobs being reached for, and that people want stiffness that holds a rest state: "Dampening alone can't keep something from moving."
- A corpus measurement (how many shipped puppets leave inochi2d physics at defaults) would settle this. The project already has a puppet corpus research file ([2026-11-26-puppet-corpus.md](2026-11-26-puppet-corpus.md)); I did not check whether it records physics values.

### 5. Constants beginners get wrong

- **Chain structure in Cubism**, not a constant: mismatched pendulum count vs output count, all outputs left on pendulum 1, and total influence above 100% on one parameter, which makes motion jerky ([editor manual](https://docs.live2d.com/en/cubism-editor-manual/physical-operation-setting/)). Nesting nodes removes the count mismatch and the all-on-one-pendulum case by construction; total influence above 100% is a parameter-binding concern.
- **Cubism `Delay` and `Acceleration`** have no physical meaning a beginner can reason about, and `Delay` is frame-rate-scaled by a magic 30. These are candidates for exclusion, which Option B would import.
- **inochi2d `model_type`** silently disables `frequency` and `length_damping` in the rigid model. This is the clearest removable knob.
- **`length_damping` vs `angle_damping`** at equal defaults (0.5) is probably a source of confusion for a hair rig, where stretch is usually not wanted. Whether to collapse them into one damping, derive the radial one from it, or keep both is **not settled by evidence**. I kept both because the spring model needs both, and a derived default is a cheap later change in the editor (not a format change); dropping a stored field later is not.
- Evidence that any specific constant is *most often wrong* is absent. This sub-question has no data behind it.

## What would change this

- **inochi2d 0.9 rework.** The source path is `nodes/legacy/`, and #479 is milestoned to 0.9. If the node's fields are reworked there, re-read before the field names are frozen. Re-check at design time.
- **The equations of motion.** I read the derivative description, not the code. If `angle_damping` is not a damping ratio in the usual sense (for example, scaled by frequency), the unit column changes. Reading `Pendulum.eval` and `SpringPendulum.eval` in `simplephysics.d` settles it; it is a short read.
- **Ranges.** Neither incumbent declares any. A stability sweep of the runtime's integrator at its fixed step over frequency, damping and length settles the upper bounds; this is a measurement, not research.
- **Whether riggers want a second axis or per-axis constants.** Would be settled by rigger-authored guides or a corpus of real physics settings. If the corpus shows many riggers stacking two nodes to get different x and y behaviour, add per-axis fields.
- **Docs I could not reach.** Live2D's own `.physics3.json` format page was not retrievable; the key list is from the framework parser. The mapping of editor labels to file fields is an inference. Re-check against the official format spec if the Cubism side is ever imported.
- **Cubism documentation pages are undated**; the version-specific note is that 4.2 and later use the FPS recorded in the file.
- **Rigger tuning behaviour** is the thinnest evidence. A moderate-budget search found no tutorial that names inochi2d constants. If the owner has rigger contacts, a handful of real files would outweigh everything in questions 4 and 5.

## Rejected options

- **B. Cubism's parameterisation.** The per-vertex factors are dimensionless, tied to a 30 fps reference, and meaningful only inside a chain. Input normalisation, weights and flips exist to drive a pendulum from parameters, which this node does not do. No stiffness knob, so the spring behaviour riggers ask for in #479 has no home. Gravity is global in the file, so a per-node gravity would have to be invented anyway.
- **C. Minimal damped pendulum (length, gravity, damping, restoring stiffness).** This is close to the chosen set. It loses because it has one damping, and the spring model needs a radial and a tangential damping to be well-behaved; and because it omits `output_scale`, the constant Live2D's own tool automates. If the owner prefers fewer knobs, the cheapest cut from the chosen set is collapsing the two dampings, which is the one place C's instinct is plausibly right and the evidence is silent.
- **A, as-is (including `model_type`, `map_mode`, `local_only`).** The model switch is a trap; `map_mode` is binding, not simulation; `local_only` is a consequence of tree placement. Adopting these stores choices that are not simulation constants.
