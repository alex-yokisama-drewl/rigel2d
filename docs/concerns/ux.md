# UX

Interaction, layout, states, and the user's path through a flow. The dimension where "it works"
and "it is usable" diverge most, and the one a developer in flow is least likely to sweep.

## Universal

### Every state exists

- **Empty state** — `test` / `judgment`
  What the user sees before there is any data. Not a blank area.
- **Loading state** — `test`
  Anything that can take time shows that it is taking time. The user never wonders whether their
  click registered.
- **Error state** — `test`
  Every failure the user can cause, and every failure they cannot, produces something that says
  what happened and what to do. Never a silent no-op.
- **Partial and slow state** — `judgment`
  What is shown when some of the data arrives and some does not.

### Interaction

- **No interactive element is covered by another** — `test`
  Bounding-box intersection between every interactive element and every piece of persistent
  chrome, at each supported breakpoint. *This is the check that was missing when a hotspot
  shipped beneath an inventory strip.*
- **Nothing overflows its container unintentionally** — `test`
  Scroll width against client width.
- **Destructive actions are confirmable or reversible** — `test`
  One of the two. Never neither.
- **Double submission is prevented** — `test`
  The second click does not produce a second record.
- **A form that fails preserves what was typed** — `test`
- **Validation says what is wrong and where** — `test`
  Inline, at the field. Not a summary that makes the user hunt.

### The journey

- **The flow can be completed without leaving it** — `judgment`
  No step requires information the user has no way to get from where they are.
- **Interruption is survivable** — `judgment`
  Closing the tab mid-flow, coming back, using the browser's back button.
- **Success is visible** — `test`
  The user can tell the thing worked without checking somewhere else.
- **Every entry point to this feature was considered** — `judgment`
  Deep link, bookmark, refresh mid-flow, arriving with stale state.

### Layout

- **Works at the supported breakpoints** — `test`
  Bounding-box assertions at each, not a glance at one.
- **Long content does not break it** — `test`
  The longest plausible name, title, list. Test with the real maximum, not a short example.
- **Text in the user's language is not hard-coded to one width** — `judgment`

## This project

<!--
Generated at planning by concern-analyst, approved by the user, grown by harvest at the end of
every cycle. Each entry carries its provenance. See README.md.
-->
