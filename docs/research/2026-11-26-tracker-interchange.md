# What must a channel mapping store for a delivered puppet to behave correctly on a recipient's tracker?

**Researched:** 2026-11-26 · **Feeds:** what a channel mapping carries, in
[performing.md](../requirements/performing.md#channel-mappings-are-puppet-data-and-live-writes-them-decided)
and the puppet format · **Verdict:** Channel name to parameter, plus an authored response curve
in the puppet, plus per-source normalisation in Live's own configuration. A name alone is not
enough, and a stored input range alone fixes the wrong half of the problem. Option 3's optional
source qualifier is already decided and nothing here disturbs it.

Every claim below is marked **[specified]** (stated in a primary source), **[reported]**
(third-party or community account), or **[inferred]** (my reasoning from something a source
states but does not itself conclude).

## Verdict

**The delivery premise survives, but only because Rigel2D writes its own protocol.** It would
not survive on any existing vocabulary. The decisive finding is not that ranges go unspecified
— several published vocabularies do specify them — but that **a specified range is necessary
and demonstrably not sufficient.** ARKit pins its coefficients to 0.0–1.0 and a real user's
hardest squint reads 0.80; VRCFaceTracking's own answer to that is a calibration pass that
re-normalises each key to how expressive *that person* is. The gap between a spec's range and a
person's range is the thing that breaks a delivered mapping, and no amount of specification
closes it.

**So the variance that matters is not between trackers. It is between people and camera
setups, and it is present with one tracker and one person who moved their webcam.** This
reframes the question. A per-puppet stored input range cannot be correct on arrival for anyone
but the rigger, because it encodes the rigger's face at the rigger's desk. Compensating for
that is per-machine work, it is identical for every puppet the recipient owns, and so it
belongs to the source in Live's configuration — Option 4. Doing it per-puppet means doing it
again for every puppet, which is the exact cost `performing.md` cites against putting mappings
in the scene.

**But Option 2 is not thereby dead, because distinct transformations wear the one name.** VTube
Studio's single stored input range conflates them, and that conflation is why its users re-tune
a purchased model:

| | Varies with | Belongs to |
|---|---|---|
| **Normalisation** — this person's neutral reads +3° of yaw, their maximum squint reads 0.80 | The person, the camera, the tracker | The source, in Live's configuration |
| **Shaping** — this puppet shuts its eye fully at 0.7; narrow the range so it moves dramatically while gaming | The puppet, and taste | The puppet, in the mapping |

Shaping is authored work a rigger does and a recipient wants delivered. Normalisation is work a
recipient does once and wants applied to everything they own. Separating them is what makes
delivery actually work rather than nominally work: the recipient calibrates their source once,
and every puppet they are handed thereafter arrives shaped and already correct.

**Therefore the mapping stores the channel name, the parameter, and a response curve defined
over the *normalised* channel** — explicitly not over raw tracker numbers. The curve is
meaningful on arrival because its input domain is a calibrated 0..1 or -1..1, not whatever the
rigger's webcam happened to produce. Without the normalisation layer underneath it, the curve
is as unportable as a raw range and should not be stored at all.

**A consequence for the protocol spec, which is the thing most at risk of being assumed.**
`performing.md` says Rigel2D defines the channel vocabulary. That vocabulary must pin, per
channel: the unit, the numeric range, the zero point, and the sign. Pinning the name and the
meaning is what VMC does and it is not enough — see the eye-openness table below, where widely
used sources disagree on every one of those properties for a single quantity. The spec must
also state the **semantic class** of each channel: whether it is an absolute physical
measurement (head yaw in degrees), a modelled 0..1 activation (ARKit-style), or a value already
relative to the user's own running baseline (OpenSeeFace-style). These are structurally
different quantities and a normalisation layer has to treat them differently — a degree
measurement should not be auto-ranged to the user's observed extremes, and a median-relative
feature must not be calibrated twice.

**Confidence.** High on Question 1, which is settled by primary sources that contradict each
other on the wire. Moderate on Question 2: the practice evidence is consistent but thin, drawn
from a rigger's own post-delivery guide and a tracking-setup guide rather than a survey. High
on Question 3 for VTube Studio and VRCFaceTracking; Inochi Session's storage I could not
determine.

## Question 1 — do different trackers provide interchangeable data?

**No. Not in unit, not in range, not in zero point, not in sign, and not even in what kind of
quantity the number is.**

### The hinge: do published vocabularies specify a numeric range?

Some do, some do not, and the ones that do specify only the range and not the rest.

| Vocabulary | Range specified? | What it says |
|---|---|---|
| ARKit blend shapes | **Yes** | "a floating point number indicating the current position of that feature relative to its neutral configuration, ranging from `0.0` (neutral) to `1.0` (maximum movement)" **[specified]** |
| VRM 1.0 expressions | **Yes** | "Value is a number with a value in the range [0-1]"; implementations *should* clamp outside it. Note VRM 0.x used [0-100] for `morphTargetBinds` weight — the same concept rescaled across a version boundary **[specified]** |
| VRChat OSC face tracking | **Yes** | One unsigned byte per key, decoded as `byte / 255.0` to a strength 0 to 1. Names and indices fixed; unsupported keys must be sent as 0 **[specified]** |
| VMC Protocol (Marionette) | **No** | `/VMC/Ext/Blend/Val (string){name} (float){value}` — a name and a float, no range, no units on bone or root transforms. Names come from the sender's own VRM, and are case-sensitive **[specified]** |
| Live2D standard parameters | **Yes**, but for the *rig's* axis | Min/default/max per parameter ID: `ParamAngleX` -30/0/30, `ParamEyeLOpen` 0/1/1, `ParamMouthOpenY` 0/0/1. Exists so "common parameters for use make it easy to replace, reuse, etc." **[specified]** |
| VTube Studio input parameters | **Mostly no** | The wiki states a range for `MouthOpen` ([0,1]) and gives none for the others. The API returns min/max/defaultValue per parameter, but states "The min/max values aren't actually the minimum and maximum values allowed for the parameter" — they are the defaults used when a new mapping is created **[specified]** |

Two things follow. First, **Live2D's specified ranges are on the parameter, not the channel** —
they pin what the puppet's axis means, which is the analogue of a Rigel2D parameter, and say
nothing about the tracker. Mapping a tracker onto them is precisely the work left over.
**[inferred]** Second, **VMC is the closest analogue to what Rigel2D is building and it pins
neither range nor vocabulary**, deferring both to the sender's model. An adapter for VMC
therefore cannot be a pass-through; it has to carry a per-sender table. **[inferred]**

### The same quantity, four conventions

Eye openness, as the clearest case:

| Source | Channel | Zero means | Other end | Range |
|---|---|---|---|---|
| ARKit | `eyeBlinkLeft` | eye open (neutral) | fully closed | 0 .. 1 |
| OpenSeeFace, `features` | `EyeLeft` | **normal open** | -1 closed, +1 wider than normal | -1 .. 1 |
| OpenSeeFace, separate field | `leftEyeOpen` | closed | open — "how likely it is that the eye is open" | 0 .. 1, no stated range |
| Live2D | `ParamEyeLOpen` | closed | open | 0 .. 1 |

Sign, zero point and range all differ, and **OpenSeeFace emits two mutually incompatible eye
numbers itself**, one a signed deviation and one a probability. A mapping that names "eye
openness" and nothing else picks a row of that table at random, including rows whose sign is
inverted relative to each other. **[specified]** for each row; **[inferred]** for the
conclusion.

### Worse: the numbers are not the same kind of quantity

OpenSeeFace's `features` are documented against a running baseline, not an absolute: `MouthOpen`
is "how open or closed the mouth is, **compared to its median pose**", `EyebrowUpDownLeft` is
position "compared to its median position". The README separately says the expression detector
"has to be calibrated on a per-user basis". **[specified]**

ARKit's coefficients are relative to a *modelled* neutral configuration, fixed by Apple's face
model, not to the user's observed median. **[specified]**

So `mouth_open = 0.5` from these two sources is not a noisier version of the same measurement;
it is a different measurement. One means "half of this person's recent range", the other "half
of the model's maximum jaw travel". **[inferred]** This is the finding most likely to be
assumed away, because both are floats in roughly the same interval.

### Head rotation has no agreed unit at all

| Source | Form |
|---|---|
| OpenSeeFace | `rawQuaternion`, plus `rawEuler` / `rotation` as Vector3. No units in comments; `% 360` in the code implies degrees **[inferred]** |
| VMC | Quaternion, no stated units for the position component **[specified]** |
| VTube Studio | `FaceAngleX` with an example min/max of -30/30 in the API response, consistent with degrees **[specified]**; a third-party guide gives the default input range as -30..30 and observes natural movement only reaches about -20..20 **[reported]** |
| ARKit | A transform matrix on the face anchor, not an angle **[specified]** |

### And a specified range is still not the range you get

| Finding | |
|---|---|
| A tracking-setup guide's author measured their own hardest squint at 0.80 against a nominal maximum of 1, and notes "we're wasting 20% of potential expressiveness by leaving this as is." The guide instructs readers to find their own figure: "When you're squinting as hard as you can, the left number is your personal maximum" **[reported]** | A person does not fill a specified range |
| A third-party tool exists whose entire function is to listen on the iFacialMocap port and measure "the maximum value of each Arkit Blendshape," so users can "finetune your blendshape ranges in your favourite tracking application" **[specified]**, in the sense that the tool's own page states this | The gap is common enough to be worth tooling |
| VRCFaceTracking's release notes: calibration "just sets the normalization (between 0-1) of parameters based on how expressive you are with the tracking. This in theory should mean that the calibration should work best with your face" **[specified]** | The incumbent's fix is per-user normalisation at the source layer |
| A third-party guide reports natural head movement reaching only about -20..20 against a -30..30 default input range **[reported]** | Same gap on an angular channel |

**Contradiction worth recording:** ARKit specifies 0.0 means neutral and 1.0 means maximum
movement, which reads as a calibrated absolute. The measured behaviour above says the attainable
maximum is person-dependent. These are not strictly inconsistent — "maximum movement" is a
property of Apple's face model, not of the user — but anyone reading the spec as a promise
about the received signal will be wrong, and that misreading is the whole risk here.

### A unit mismatch on a vocabulary that *did* specify one

iFacialMocap transmits ARKit's blendshape set as integers 0–100, per a third-party Unity client
that parses the protocol; I found no vendor wire-format specification. **[reported]** Other
consumers of the same ARKit names use 0.0–1.0 **[specified]**. Unity's own `BlendShapeWeight`
documents 0.0–100.0; VRChat's OSC path uses `byte / 255.0`. **[specified]** So even where the
source vocabulary pinned a range, transports carrying it rescaled it, under the same names.

## Question 2 — what do riggers actually do?

**They rig for a target tracker, they do ship a configured mapping with the model, and the
recipient re-tunes the input ranges anyway.** The evidence is consistent across the sources
found but is not large; treat it as directional.

A rigger's own post-delivery checklist for a commissioned VTube Studio model **[reported]**:

- **Most settings arrive pre-configured.** The recipient is not told to build the mapping; the
  mapping is there.
- **The recipient must still set the input ranges themselves.** Set the camera in its streaming
  position first, then "adjust the upper and lower limits of the IN value" so the neutral
  reading sits centred — listed for face position, body rotation, face tilt and head rotation.
  The trigger given is "if your body or face is tilted even when in the neutral position."
- **Sensitivity is per person.** Eyebrows "require high sensitivity settings like 0.55 to 0.65
  to move properly" for some users; webcam users are told to check sensitivity separately.
- **The target tracker is assumed, and stated.** "Models are basically set up for iPhone and
  webcam tracking." For Android, "adjust it yourself." Tracking quality is ranked iOS, then
  webcam, then Android.
- **The settings file is a deliverable both ways.** Users are told to send their `.vtube.json`
  back to the rigger when ordering additions, because a new outfit overwrites it.

Corroborating:

- A tracking-setup guide frames the whole job as personal fit — tracking should be set up "to
  suit your features and expressions" — and walks the reader through measuring their own
  per-parameter maximum and minimum. **[reported]**
- Commissioning guidance disagrees on deliverables: one says the rigger's export packages the
  model "often .vtube.json or compatible folder," another lists only `.moc3`, `.model3.json`
  and `.physics3.json` and advises asking explicitly for the VTube Studio files. **[reported]**
  So whether the mapping ships at all is not universal practice in that ecosystem.

**What this does to the portability argument.** It weakens the "unknown recipient tracker"
framing and strengthens the design. Riggers already treat the tracker as known — so
cross-tracker portability is a smaller problem than it looked. But they *also* cannot deliver a
finished result even to a recipient on the same tracker, because the remaining variance is the
person and the camera. The work that survives delivery is the shaping; the work that does not
is the normalisation. That is the split the verdict turns on, and it came from the practice
evidence rather than from the specifications. **[inferred]**

## Question 3 — what do the incumbents store, and where?

| Tool | Where the assignment lives | Does it carry range or calibration? |
|---|---|---|
| **VTube Studio** | `<Live2D-Model-Name>.vtube.json`, "right next to your Live2D model," created automatically, human-readable JSON, not encrypted **[specified]**. The wiki states there is no documentation of the format **[specified]** | Yes — the mapping holds both input and output ranges, a Limit Range option, and smoothing **[specified]**. Field names unverified: I did not open a sample file |
| **VTube Studio, API view** | — | Per-parameter `min`, `max`, `defaultValue`, which are "the default lower and upper values used when a new parameter mapping is created," not enforced limits **[specified]** |
| **VRCFaceTracking** | Per-user calibration, applied as a "data mutation" step in the tracking application, between the source and the avatar **[specified]** | Yes — this is purely calibration, re-normalising each parameter to 0–1 against the user's expressiveness. Not stored with the avatar |
| **VRChat OSC** | Avatar parameters by fixed name and index; the transport specifies 0–1 **[specified]** | No calibration in the protocol — it sits upstream, in VRCFaceTracking |
| **Inochi Session** | **Could not determine.** Not in the docs reachable within budget; would need the source repository | Unknown |

Two details from VTube Studio that bear directly on the decision:

- **Auto-setup derives the mapping from naming convention, not from delivery.** Loading a new
  model prompts an auto-setup that "will set up your model based on standard Live2D parameter
  names and values," referencing Live2D's standard parameter list; renaming parameter IDs or
  changing their ranges defeats it, and manual fine-tuning may still be needed afterwards.
  **[specified]** So the incumbent's answer to "does a delivered rig move on arrival" is
  substantially *convention over standard parameter names*, with the stored file as a refinement
  on top. Rigel2D has no equivalent fallback: a puppet's parameters are author-named and carry
  no standard IDs, so the stored mapping is load-bearing in a way VTube Studio's is not.
  **[inferred]**
- **VTube Studio's own advice contradicts the rigger's.** The wiki: "it is recommended to only
  change the output range and leave the input range as is" — i.e. treat the input range as a
  property of the tracker and the output range as the per-puppet knob. The rigger's guide tells
  recipients to adjust the IN values to centre their neutral. **[specified]** and **[reported]**
  respectively. Both are right about different things, and the disagreement is the
  normalisation/shaping conflation surfacing as user confusion. It is the clearest argument in
  the whole report for separating the two rather than storing one number.

Also notable: VTube Studio ships a **copy-config-between-models** feature that transfers the
tracking-to-model parameter mapping wholesale, existing "for users with several models that
share similar Live2D parameters." **[specified]** That is users reaching for per-machine reuse
of mapping work through a per-model storage design — the shape Option 4 provides natively.
**[inferred]**

## What would change this

- **Opening a real `.vtube.json`** would confirm the field names and whether the input range is
  stored per mapping or per input parameter. I reasoned from the wiki's prose, not the file.
  Cheap to settle and worth doing before the format is fixed.
- **Inochi Session's storage location is unknown.** It is the direct comparator and would be
  worth half an hour in its repository, though `performing.md` already declines to read its
  formats, so it informs the design and not compatibility.
- **No vendor wire-format specification for iFacialMocap was found.** The 0–100 integer claim
  rests on a third-party client. It does not change the verdict — the ARKit/VRM/VRChat/Unity
  rescalings are enough on their own — but do not cite the iFacialMocap figure as primary.
- **The practice evidence is two guides, not a survey.** They agree, and the rigger's checklist
  is specific enough to be credible, but a contrary finding — that recipients typically change
  nothing — would move the shaping/normalisation line. Worth testing against riggers directly if
  the question ever gets expensive.
- **Re-check ARKit and VRM if either versions.** VRM already rescaled 0-100 to 0-1 across a
  major version; a vocabulary's range is not a stable thing to depend on, which is itself an
  argument for Rigel2D normalising at the adapter rather than trusting what arrives.
- **What would genuinely reverse the verdict:** evidence that auto-calibration at the source is
  unacceptable in practice — that users cannot or will not perform a calibration pass, or that
  auto-ranging produces unstable output on noisy channels. That would push the whole burden back
  onto per-puppet stored ranges and make Option 2 alone the only workable answer despite its
  per-puppet cost. Nothing found suggests it, and VRCFaceTracking shipping exactly this is
  evidence against, but it has not been tested here with Rigel2D's beginner audience in mind.

## Rejected options

**Channel name to parameter, and nothing more.** Rejected because a conforming source still
cannot know a given user's attainable range or resting offset. ARKit specifies 0.0–1.0 and a
real squint reads 0.80; a nominal -30..30 head yaw sees about -20..20 in natural movement. A
mapping with no normalisation under it delivers a puppet that moves but under-reacts, which is
the failure mode hardest to diagnose because nothing looks broken. The option is not wrong about
the protocol — Rigel2D should pin unit, range, zero and sign per channel — it is wrong that
pinning them is sufficient.

**A stored input range in the puppet, as the whole answer.** Rejected because it stores the
rigger's face and the rigger's camera placement in the recipient's file. It is per-puppet work
the recipient must redo for every puppet they own, which is the cost `performing.md` already
rejects when it refuses to put mappings in the scene. VTube Studio implements exactly this and
its own users are instructed to re-tune the input ranges on arrival. Retained only in the
narrowed form the verdict describes: a response curve over the *normalised* channel, which is
shaping rather than compensation.

**A source reference as the mechanism for correctness.** Rejected for that purpose — naming the
rigger's source tells the recipient's player nothing it can act on, since the recipient does not
have that source. Not rejected as a feature: `performing.md` already decides a mapping may
optionally name a source for disambiguation when two sources supply one channel name. That
decision stands and is orthogonal to this one.

**Per-machine normalisation alone, with nothing stored in the puppet beyond the name.** Rejected
because it discards authored shaping. "This puppet's eye shuts fully at 0.7 of normalised input"
and "narrow this axis so the puppet reads dramatically" are rigging decisions with no per-machine
meaning, and a rigger who cannot ship them ships a puppet that moves correctly and looks wrong.
Normalisation is necessary; it is not the whole mapping.

## Sources

Primary specifications:

- Apple, `ARFaceAnchor.blendShapes` — <https://developer.apple.com/documentation/arkit/arfaceanchor/blendshapes>
- VRM 1.0 expressions — <https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/expressions.md>
- VMC Protocol, Marionette spec — <https://protocol.vmc.info/marionette-spec>
- VRChat OSC face tracking — <https://creators.vrchat.com/avatars/face-tracking/osc>
- Live2D standard parameter list — <https://docs.live2d.com/en/cubism-editor-manual/standard-parameter-list/>
- OpenSeeFace `OpenSee.cs` (feature definitions and packet fields) — <https://github.com/emilianavt/OpenSeeFace/blob/master/Unity/OpenSee.cs>
- OpenSeeFace README (per-user calibration) — <https://github.com/emilianavt/OpenSeeFace>
- VTube Studio, VTS Model Settings — <https://github.com/DenchiSoft/VTubeStudio/wiki/VTS-Model-Settings>
- VTube Studio, Model File — <https://github.com/DenchiSoft/VTubeStudio/wiki/VTube-Studio-Model-File>
- VTube Studio, Copy config between models — <https://github.com/DenchiSoft/VTubeStudio/wiki/Copy-config-between-models>
- VTube Studio API (`InputParameterListRequest`, min/max semantics) — <https://github.com/DenchiSoft/VTubeStudio>
- VRCFaceTracking releases (calibration as data mutation) — <https://github.com/regzo2/VRCFaceTracking/releases>
- iFm Checker (measures received blendshape maxima) — <https://felineentity.itch.io/ifm-checker>

Third-party and community:

- Rigger's post-delivery setup guide for a commissioned VTube Studio model — <https://note.com/ramusika/n/n1fef9688e7e2?hl=en>
- Face tracking and VBridger setup guide (personal maxima, the 0.80 squint) — <https://www.bilvyy.com/post/vbridger>
- UnityIFM, third-party iFacialMocap client (0–100 integer values) — <https://github.com/Menithal/UnityIFM>
- Live2D parameter binding guide (default input ranges, -20..20 natural movement) — <https://0x-live2d-models-website.pages.dev/docs/advanced/parameter-binding>
