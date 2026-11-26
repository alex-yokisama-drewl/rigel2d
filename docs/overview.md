# Overview

What this project is, and what it is deliberately not.

## What it is **[proposed]**

A free and open-source toolkit for 2D vtubing: an application for rigging a layered drawing into
an animated puppet, and an application for performing that puppet live from face tracking into
streaming software. It occupies the same ground as Inochi Creator and Inochi Session, and exists
because that ground is covered by tools that are capable but unwieldy, or polished but paid.

It is aimed at smaller streamers and at beginners. Larger channels tend toward 3D or toward the
paid tools they already know; the people underserved are the ones for whom the existing free
option is a wall of unintuitive interface, and for whom the hardware is a laptop.

## What it is not **[proposed]**

- **Not a feature race.** The trade is explicit: cover the basics excellently rather than cover
  everything. A capability that cannot be made intuitive is a candidate for exclusion, not for a
  tooltip.
- **Not inochi2d-compatible.** Its deformation model is the starting point and its file formats are not —
  nothing here loads or saves `.inp`, in either direction ([decisions.md](decisions.md)).
- **Not a tool that assumes a good computer.** Running well on weak hardware, and leaving most
  of that hardware to the game and the encoder, is a specified budget rather than an aspiration
  ([requirements/platform.md](requirements/platform.md)).
- **Not two independent products.** The two applications share a format and a runtime, and are
  specified together, because a rig that looks right while being authored and wrong while being
  performed is the failure this structure exists to prevent.
- **Not a 3D tool, a face tracker, or streaming software.** It consumes tracking and feeds
  streaming software; it implements neither.

## Shape **[proposed]**

| Piece | What it is | Where it is specified |
|---|---|---|
| The puppet | The format and the deformation model both applications share | [requirements/puppet.md](requirements/puppet.md), [format.md](format.md) |
| The runtime | Evaluates and renders a puppet. Embedded in both applications, so that preview and performing are one code path | [runtime.md](runtime.md), [evaluation.md](evaluation.md) |
| The editor | Builds puppets | [requirements/authoring.md](requirements/authoring.md) |
| The player | Performs puppets live | [requirements/performing.md](requirements/performing.md) |

Build order is runtime, then editor, then player. That is an order of construction and not of
specification — see [decisions.md](decisions.md).
