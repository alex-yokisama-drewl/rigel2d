# Accessibility

Keyboard, assistive technology, contrast, motion. Almost entirely mechanically checkable, which
makes it the dimension with the least excuse for `judgment` entries.

## Universal

### Keyboard

- **Every interactive element is reachable by keyboard** — `test`
  Tab through the flow. Nothing is mouse-only.
- **Focus order follows visual order** — `test`
- **Focus is always visible** — `test`
  A focused element has a visible indicator that is not removed by a style reset.
- **Focus is trapped in a modal and restored on close** — `test`
  Returns to the element that opened it.
- **Escape closes what it opened** — `test`

### Assistive technology

- **Every control has an accessible name** — `test`
  Accessibility tree, not a visual label that is not associated.
- **Every image has alt text, or is marked decorative** — `test`
- **Form fields are associated with their labels and their errors** — `test`
- **Dynamic changes are announced** — `test`
  A live region for anything that appears without a navigation.
- **Heading levels are sequential** — `script`
  No jump from h1 to h3.
- **Landmarks exist** — `script`

### Visual

- **Text contrast meets the project's stated level** — `test`
  Computed colour, both themes if there are two. Never judged from a screenshot.
- **Non-text contrast** — `test`
  Borders, icons, focus indicators, chart series.
- **Nothing is conveyed by colour alone** — `judgment`
  A red border also carries text.
- **Interactive targets meet the minimum size** — `test`
  Bounding box.
- **The layout survives text scaling** — `test`
  At 200%, at each breakpoint.

### Motion

- **`prefers-reduced-motion` is honoured** — `test`
- **Nothing flashes more than three times a second** — `judgment`
- **No motion is required to understand state** — `judgment`

## This project

<!--
Generated at planning, approved by the user, grown by harvest. See README.md.
Record the project's target level and any exemptions here on the first planning sweep.
-->
