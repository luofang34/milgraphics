# Changelog

## 0.1.2 — 2026-10-11

Terrain occlusion for the screen tier.

- `Projection::project_depth` lets a host give, with each screen position,
  the depth its frame stores there. Every point of every screen item then
  carries that depth (`ScreenItem::depth`), and labels, handles and symbol
  placements carry their anchor's (`depth` on each), so a host can test the
  screen tier against its frame's depth and let terrain hide what lies
  behind it. Hosts that give no depth see no change.
- Plans for views of many graphics are quicker to build.

## 0.1.1 — 2026-10-11

Graphics on terrain and in tilted views.

- `Projection::max_chord_px` lets a host that projects onto terrain bound
  the chords drawn without sampling the ground, so lines follow it.
- Pixel-sized decorations (teeth, scallops, wire, arrowheads) and the range
  fan's pointer lie on the ground and are cut at the horizon, rather than
  running straight between the control points' screen positions.
- With a viewport, the screen tier is cut a viewport's size past it and
  every screen position is finite, whatever the camera.
- Stacked labels are a host's line height apart
  (`FontMetrics::line_height_px`) and keep their pixel spacing and their
  distance from the line in tilted views (`Label::offset_em`).
- `Construction::has_screen_decorations` tells a host which graphics have
  decorations only the screen tier draws.

## 0.1.0 — 2026-10-10

First release: native Rust multi-point tactical graphics.

- Every multipoint control measure and METOC line and area of MIL-STD-2525D
  change 1 (version code 11) and MIL-STD-2525E change 1 (15) that the
  standard defines, and of version codes 10 and 16: 1,495 declarations,
  each with its control points, amplifiers and standard reference in
  `support::all()` for palettes and amplifier forms.
- Drawn as the standard's templates show them: every graphic is compared
  with the pinned mil-sym-java oracle under a stated tolerance, or, where
  the standard differs from it, with a golden reviewed against the template
  (listed in `tests/fixtures/oracle/standard.txt` and UPSTREAM.md). Codes
  the standard does not define for an edition are typed errors.
- `construct` builds a geographic construction once per definition;
  `render` turns it into a plan for one view: geographic tier (WGS84,
  antimeridian-split), screen-space decorations, labels, embedded
  single-point symbol placements (drawn by the host, e.g. with milsymbol),
  edit handles and pick references. Projection and font metrics are
  injected, so the same code serves globe, perspective and flat views.
- Pure edits (`apply_edit` with an `EditContext`) that honour each
  symbol's constraints and return the new construction; amplifier edits
  through the same path. `PersistedGraphic` is the only persisted form and
  keeps unknown fields and unknown versions intact.
- GeoJSON for map engines with documented, stable property names and
  values, one dash pattern per layer (`DashPattern::ALL`).
- Budgets for input and output size, typed errors for every limit, and
  screen-sized patterns bounded and steady at any zoom.
- Native and `wasm32-unknown-unknown`, Rust 1.85 or later; no JavaScript,
  JVM, map engine, GPU or file system at build or run time.
