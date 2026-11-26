# Research

Dated findings, each against the decision it fed.

## Rules **[decided]**

- **One file per question**, named for the question, not the topic.
- **Every file carries its date and the decision it fed** in a header line. Research stales on a
  different clock than design does: a pricing answer or an API capability from months ago is a
  liability sitting beside design decisions that are still true. The date is what lets a reader
  tell at a glance.
- **A research file is not a decision.** The decision lives in the design document that states
  the resulting rule, and in [../decisions.md](../decisions.md) only if it meets that document's
  admission test. The decision links here; this links nowhere.
- **An unlinked research file is re-researched.** Link it from wherever the decision landed.
- **Pruned at the end of a cycle** when stale and no longer feeding a live decision.
  `doc-reviewer` flags candidates by date every cycle.

## Format

See `.claude/agents/researcher.md`, which writes these files. In short: verdict first, then the
evidence with dated sources, then what would change the conclusion, then the rejected options
with the specific reason each lost.

## Files

<!-- Listed in ../README.md's index like any other document. -->

| Finding | Feeds |
|---|---|
| [2026-11-26-linux-alpha-output.md](2026-11-26-linux-alpha-output.md) | How the player delivers alpha-channel frames into OBS on Linux |
| [2026-11-26-inochi-file-measurements.md](2026-11-26-inochi-file-measurements.md) | What is actually inside an inochi2d model file, measured on real rigs |
| [2026-11-26-deformation-models.md](2026-11-26-deformation-models.md) | How other 2D systems model deformation, and whether a skeleton is needed |
| [2026-11-26-puppet-corpus.md](2026-11-26-puppet-corpus.md) | How large a real 2D vtuber puppet is, measured across free Live2D and inochi2d rigs |
