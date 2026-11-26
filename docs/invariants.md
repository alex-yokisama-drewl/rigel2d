# Invariants

What must never be broken, and what a violation looks like.

This document and the invariant list in `CLAUDE.md` are **one list in two halves and must never
disagree**: CLAUDE.md carries the rule and its reason in one line; this document carries the
failure mode, numbered to match. `scripts/docs-check.sh` checks that the numbers correspond.

A violation found in review outranks every other finding.

## Format

Each entry:

```markdown
## I<n>. <The rule, stated as what must always be true>

**Why:** <what the rule protects. A rule without one gets "fixed".>
**Violation looks like:** <what you would observe. Concrete.>
**Checked by:** <a test, a script, or "judgment" — and where it lives.>
```

An invariant with no check is a hope. Where one cannot be automated, say so explicitly rather
than leaving the line blank.

## Invariants

<!--
Added during tech-design and as subsystems are built. Numbering is stable: a retired invariant
keeps its number and is marked retired rather than renumbered, because CLAUDE.md and review
reports reference these numbers.
-->

*None yet.*
