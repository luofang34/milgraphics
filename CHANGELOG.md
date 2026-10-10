# Changelog

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
