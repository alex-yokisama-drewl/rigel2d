# UI assertion patterns

Deterministic checks for things normally eyeballed. Guidance rather than a library, because the
harness differs per project — the examples use Playwright and translate directly to anything
that exposes the DOM and computed styles.

**Assert on structure, never on pixels.** A screenshot parsed by a model is expensive,
non-deterministic, and misses what it is reached for. Screenshots are for a person to look at.

## Overlap

The case this suite was designed around: an interactive element rendered beneath persistent
chrome, shipped, and found by a human playing the game. A bounding-box intersection assertion
catches it on the first run and costs nothing thereafter.

```js
const intersects = (a, b) =>
  a.x < b.x + b.width && b.x < a.x + a.width &&
  a.y < b.y + b.height && b.y < a.y + a.height;

test('no interactive element is covered by persistent chrome', async ({ page }) => {
  await page.goto('/scene/1');
  const chrome = await page.locator('[data-chrome]').all();
  const chromeBoxes = await Promise.all(chrome.map(c => c.boundingBox()));

  for (const el of await page.locator('[data-interactive]').all()) {
    const box = await el.boundingBox();
    if (!box) continue;
    for (const c of chromeBoxes) {
      expect(c && intersects(box, c), `${await el.getAttribute('data-interactive')} is under chrome`)
        .toBeFalsy();
    }
  }
});
```

Run it at every supported breakpoint. Overlap is a function of width, and the version that
passes at one size is the version that ships broken at another.

**This check needs a convention**, and the convention is the real work: a marker attribute on
everything interactive and everything persistent. Establish it during `tech-design` and record
it in `docs/testing.md`, or the assertion silently covers a shrinking fraction of the interface.

## Overflow

```js
const overflows = await el.evaluate(e =>
  e.scrollWidth > e.clientWidth + 1 || e.scrollHeight > e.clientHeight + 1);
expect(overflows).toBe(false);
```

Test with the longest plausible content, not a short example. The real maximum is where this
fails.

## Contrast

```js
const { fg, bg } = await el.evaluate(e => {
  const s = getComputedStyle(e);
  return { fg: s.color, bg: s.backgroundColor };
});
expect(contrastRatio(fg, resolveBackground(bg, e))).toBeGreaterThanOrEqual(4.5);
```

`resolveBackground` has to walk ancestors when the element's own background is transparent —
the commonest reason a naive contrast check reports a false pass. Write it once, in a helper,
and use it everywhere. Check both themes where there are two.

## Computed style

For "is it the right colour, size, position", assert the computed value, not a screenshot:

```js
await expect(el).toHaveCSS('background-color', 'rgb(17, 24, 39)');
```

Prefer asserting against the design token's resolved value over a literal, so a token change
fails in one place rather than everywhere.

## Keyboard and focus

```js
test('focus order follows visual order', async ({ page }) => {
  const order = [];
  for (let i = 0; i < 10; i++) {
    await page.keyboard.press('Tab');
    order.push(await page.evaluate(() => document.activeElement?.dataset.testid));
  }
  expect(order).toEqual(['search', 'filter', 'results', /* … */]);
});
```

Also: focus is visible, focus is trapped in a modal and restored on close, Escape closes.

## Accessibility tree

```js
const snapshot = await page.accessibility.snapshot();
```

Assert on names and roles. This is the check that catches a control with no accessible name —
invisible to every visual check and to a screenshot.

An automated audit library (axe and equivalents) covers much of `docs/concerns/accessibility.md`
mechanically. Run it as a test, and treat its findings as failures rather than a report.

## Breakpoints

```js
for (const width of [360, 768, 1024, 1440]) {
  test(`layout at ${width}`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    // overlap, overflow, and the layout assertions that matter at this size
  });
}
```

The widths belong in `docs/testing.md`, named once and imported — not repeated per test file.

## What stays human

Aesthetic judgment, whether a flow feels right, whether copy reads well, whether an animation
is pleasant. Send these to a gate, and **say what to look at and what would count as wrong**.
A gate that says "check the page" wastes the gate.
