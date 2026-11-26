# Analytics

What is measured, and whether a question can be answered after the fact. The dimension most
reliably forgotten, because nothing breaks when it is missing — it simply becomes unanswerable
later, when answering it matters.

## Universal

### Coverage

- **Every user-initiated action that matters emits an event** — `test`
  Decided at planning, not at implementation. "Matters" means someone will ask how often it
  happens.
- **Every flow emits start, success and failure** — `test`
  A funnel with no failure event cannot distinguish abandonment from breakage, which is the
  whole question.
- **Failure events carry the reason** — `test`
  A failure count with no cause is a number nobody can act on.
- **Every error surfaced to a user is recorded** — `test`

### Shape

- **Event names follow the project's convention** — `script`
  Stated in this file's *This project* section on the first sweep. Inconsistent naming is
  discovered months later, in a dashboard.
- **Properties are typed and consistent across events** — `script`
  The same concept has the same property name everywhere.
- **Every event carries what identifies the actor and the session** — `test`
- **No event carries a secret, a credential, or anything the privacy rules exclude** — `script`
  Automatable as a deny-list over emitted property names, and worth automating.

### Usefulness

- **Each event names the question it answers** — `judgment`
  An event instrumented with no question behind it is noise that costs money to store.
- **The question can actually be answered from what is emitted** — `judgment`
  Enough dimensions to segment by. The commonest failure is an event that proves something
  happened but not to whom, from where, or how often per user.
- **A change to an existing event is additive, or versioned** — `judgment`
  Changing an event's meaning silently invalidates every historical comparison.

### Privacy

- **Personal data in events is deliberate and recorded** — `judgment`
  Cross-reference `security.md` and `data.md`.
- **Consent is respected before anything is emitted** — `test`
  Where the project has a consent mechanism.

## This project

<!--
Generated at planning, approved by the user, grown by harvest. See README.md.
Record the naming convention, the analytics sink, and the required properties here on the first
planning sweep — the Universal entries above reference them.
-->

## This project

- **Nothing is measured, because nothing is sent** — `test`
  This project makes no outbound network connection at all (`docs/decisions.md`), so every
  Universal entry above about emitting events, naming them consistently, or answering a
  question later is **inapplicable here** rather than merely unmet. Do not raise them at
  review, and do not add an event to satisfy one.
  The question "could we answer this later?" still applies, and its only available answer is a
  local log the user can read and choose to send with a bug report.
  Seeded at planning, 2026-11-26.
