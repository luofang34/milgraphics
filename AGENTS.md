Implement **`milgraphics`**: a native, pure-Rust library for multi-point military tactical graphics (MIL-STD-2525 / APP-6 control measures and METOC lines/areas).

It is the sibling of [`milsymbol`](https://github.com/luofang34/milsymbol), which renders single-point symbols. Applications depend on both; neither depends on the other.

## Mission

Turn a persisted, editable graphic definition (symbol identity, geographic control points, typed modifiers) into map-engine-neutral output: geometry, labels, embedded-symbol placements, edit handles and pick references.

Native applications, browser WASM applications, headless servers, test tools and exporters share one implementation of symbol construction, label layout and editing rules, so that all clients agree on what a graphic means and how it looks.

The final drawing belongs to the consuming map engine. The primary consumer is Sokoly-App, which draws through the `maplibre-rs-experimental` fork (headless map sharing egui's wgpu device, free-globe navigation). See "Consumer integration".

## Relationship to milsymbol

| Library | Input | Output |
|---|---|---|
| `milsymbol` | SIDC, icon style, text modifiers | One single-point symbol |
| `milgraphics` | Symbol identity, geographic control points, modifiers, view parameters | Multi-point graphic: geometry, labels, handles, symbol placements |

- `milgraphics` must not depend on `milsymbol`. It must build and test without it.
- When a graphic embeds a unit or point symbol, emit a `SymbolPlacement` (SIDC, modifiers, geographic anchor, screen offset, size, rotation). The application renders it with `milsymbol`. The two libraries compose through data.
- An optional adapter feature may be added later, but only after the same integration appears in several consumers. Do not create a third "common symbology" crate in advance.
- Follow `milsymbol`'s engineering conventions (lint table, CI layout, oracle pinning, provenance documents, NOTICE format) unless this file says otherwise.

## Consumer integration

The display contract is shaped by [maplibre-rs-experimental](https://github.com/luofang34/maplibre-rs-experimental) as used by Sokoly-App. Design the output so it maps onto that engine without loss, but never depend on it. Real integration is part of the first milestone, not follow-up work: SVG cannot show horizon occlusion, label collision, screen-space decorations, picking or dragging.

**Where code lives**

- `milgraphics`: definitions, persistence envelope, construction, render plans, edits. No map, GPU or UI types.
- Sokoly-App (e.g. `app/src/map/graphics.rs`, next to its `ground_routes.rs` and `symbols.rs`): the adapter that turns a `RenderPlan` into GeoJSON sources, style layers, images and overlay drawing; the `FontMetrics` implementation; numeric pick IDs; persistence in the session journal.
- The fork: general engine capabilities only. Tactical-symbology code never goes into the fork.

**What the engine does and does not do, and what that requires of the output**

- *Geometry.* Committed graphics are drawn as a GeoJSON source plus line/fill/symbol/circle layers (`Map::set_geojson_data`, `update_geojson_data`). The engine does not clip or split geometry at ±180°, so the geographic tier is WGS84 degrees, already split at the antimeridian, with ring orientation and closure stated.
- *Earth model.* The engine renders on a sphere (radius 6,371,008.8 m) in Mercator, globe and vertical-perspective projections. Construction is ellipsoidal (WGS84). The difference (up to about 0.3 % of a distance) is a known tradeoff, bounded by a test for the six first-milestone graphics. It does not shift output against the basemap, since both are WGS84 coordinates placed on the same sphere; it only appears where metres are converted to degrees.
- *Dashes.* `line-dasharray` is zoom-dependent only and measured in multiples of line width. Dash patterns are therefore a small enumerated set so the adapter can use one layer per pattern. Never encode a dash as per-feature data that the engine cannot honour.
- *Decorations.* The engine has no line-decoration primitive (teeth, ticks, arrowheads, scallops). Decorations sized in geographic units go in the geographic tier as polygons/polylines. Decorations sized in screen units go in the screen-space tier for a host overlay or render pass. Each decoration states its tier.
- *Text.* Labels are emitted as point or along-line anchors with text, font stack, size, anchor, offset, rotation, and whether collision may hide them. The engine's text-width rule (glyph-PBF advances, no kerning) belongs in the adapter's `FontMetrics` implementation, not in the core.
- *Icons.* `SymbolPlacement` maps onto the engine's `StyleImageProvider` (`namespace:id` images rendered on demand, with an anchor) or a host overlay. The app renders the symbol with `milsymbol`.
- *Picking.* The core emits semantic `PickRef { definition, part, handle }` values. The adapter assigns numeric feature IDs, keeps the ID → `PickRef` table, and binds it to the render batch that produced it so stale picks are rejected. IDs that cross JavaScript `Number` stay within ±(2⁵³ − 1). The core never packs references into integers.
- *Drag preview.* A GeoJSON update re-indexes the whole source and re-requests every loaded tile, so it suits committed edits only. Previews use the screen-space tier, recomputed per frame without touching map sources.

**Fork gaps** (separate pull requests to the fork, scheduled inside the first milestone as the integration needs them): public Mercator geo → screen; refresh scoped to one source; an overlay or render-node API for host-drawn screen-space primitives; terrain elevation lookup outside headless mode; optionally data-driven dash arrays.

**Host persistence gap.** Sokoly-App's journal deserializes an unknown payload kind into `Payload::Unknown`, which discards its content and makes the journal read-only. That does not preserve unknown graphic data. The integration therefore includes a host change that stores unknown payloads verbatim (e.g. as `RawValue`) and writes them back unchanged.

**Toolchain.** Consumers build on Rust 1.97.1 (edition 2021) and check `wasm32-unknown-unknown` in CI; the fork pins `wasm-bindgen = "=0.2.129"`, which fixes the app's lockfile to the same version. `milgraphics` keeps `rust-version = "1.85"`, builds for `wasm32-unknown-unknown` without `wasm-bindgen`, `getrandom` or threads in its normal dependencies, uses `thiserror` 2, and exposes only plain data in its public API: its own validated `GeoPoint`, `[lon, lat]` pairs and pixel points, and no `cgmath`, `lyon`, `wgpu` or `geo` types. Conversions to `geo-types` may be added behind a feature when a consumer needs them. Browser test tooling (`wasm-bindgen-test`) is a dev-dependency only and matches the consumers' `wasm-bindgen` pin.

## Non-negotiable runtime constraints

- No JVM, Node, JavaScript engine, browser DOM or remote rendering service at runtime, at build time, or in downstream applications. JVM/Node are allowed only inside explicit development tooling under `tools/`.
- The library has no dependency on windowing, networking, the file system, GPU, `egui`, `wgpu`, MapLibre or any other map engine. CI enforces this with a dependency-tree guard (like `milsymbol`'s `tools/ci/js-guard.sh`) that bans JS-runtime, JVM-bridge, GUI, GPU and map-engine crates from normal dependencies.
- Native and browser WASM are the required targets. `std` is allowed. `no_std + alloc` is a goal to evaluate after the dependency set is settled; it is not a reason to replace a vetted dependency.
- No process-global mutable state, `static mut`, global registries, or `OnceLock`/`lazy_static!` with side effects. All configuration (standard version, DPI, label policy, style) is explicit and safe for concurrent use.
- No network access, no hidden I/O, no threads.
- Determinism: for the same inputs, every target produces the same semantics, item order and identifiers, and coordinates within a stated tolerance. Bit-identical floating-point output across targets is not promised: platform math libraries differ by a few ULPs (native and `wasm32` geodesics already do). Within one target, output is bit-for-bit repeatable.
- User input never panics, never loops forever, and never allocates without bound. Control-point counts, generated vertex counts, decoration repetitions and label counts have explicit, documented budgets that produce typed errors.

## Standards and coverage

The authorities on symbol meaning and appearance are the standards themselves (MIL-STD-2525D change 1, MIL-STD-2525E change 1, APP-6D, APP-6E). A reference implementation's output is never proof of conformance.

**Coverage is declared per (standard version, symbol code), never per standard.** The same entity code can differ between versions (Mission Command remaps some line types by version, e.g. in `clsRenderer.getCMLineType`).

Initial scope, by Mission Command version code:

| Code | Standard | Status |
|---|---|---|
| 11 | MIL-STD-2525D change 1 | Primary target |
| 15 | MIL-STD-2525E change 1 | Primary target |
| 10 | APP-6D (upstream also maps base 2525D here, deprecated in favour of 11) | Declared per symbol on oracle agreement alone (the text is unavailable); some entries exist only in 10 (e.g. Decision Line 110500) |
| 16 | APP-6E | Declared per symbol. Upstream's README says "icons only", but its `mse.txt` has line/area entries. |

Legacy 2525B/C letter codes are out of scope unless a `2525C → D` conversion is explicitly requested.

Rules:

- The generated catalog lists what upstream knows. It is not a support claim. Support is declared symbol by symbol, after that symbol's acceptance tests pass.
- Each supported symbol declares, in exactly one place, its control-point constraints (min/max count, roles, required modifiers such as `AM`/`AN`), its supported modifiers, which dimensions are geographic and which are screen-space, its handle contract, and its standard reference (document and section).
- APP-6(D) (code 10) is the exception while its text is unavailable: a symbol is declared when it matches the pinned oracle, carries no standard reference, and uses upstream's catalog draw rule. A symbol upstream draws only as its bare control points is not declared.
- Unsupported (version, symbol) combinations and unsupported modifiers return typed errors. Never fall back to a generic line or arrow that means something different.

## Data model and pipeline

```text
PersistedGraphic    envelope: schema version + typed fields + unknown content kept intact
      │  decode + validate (fails → kept as unsupported, never rewritten)
GraphicDefinition   validated typed view: the authority for construction and editing
      │  per-symbol rules
Construction        geographic skeleton: lines, areas, widths, decoration rules
      │  Projection + view + FontMetrics
RenderPlan          geographic tier + screen-space tier, labels, SymbolPlacements, handles, PickRefs
      │
map adapter (application side)
```

**`PersistedGraphic`** is the persistence envelope. It carries the schema version and either a decoded definition plus any fields this version does not understand, or, for an unknown schema/standard version, the original content untouched.

- "Lossless" has two levels, and each field states which it gets. Unknown fields of a known schema are preserved as **JSON content** (same values after parse; key order and number spelling such as `1.10` may change). Definitions with an unknown schema or standard version are preserved as **exact bytes**.
- Unsupported definitions are reported as unsupported, displayed as such by the host, and written back unchanged. They are never dropped or rewritten.

**`GraphicDefinition`** holds: stable ID; standard and version; symbol identity (full SIDC or set + entity code); ordered geographic control points, each with an optional altitude and explicit vertical datum; typed modifiers (`T`, `T1`, `H`, `W`, `W1`, `AM`, `AN`, `X`, …) with their units; style overrides; validity time; revision.

- Derived polylines, triangulations, SVG and label positions are caches, never persisted as authority.
- GeoJSON is an exchange/adapter format, not the internal model: it cannot express control-point semantics, editing rules or screen-space decoration.

**`Construction`** is computed in geographic space and depends only on the definition and configuration. It is cacheable across view changes.

**`RenderPlan`** has a geographic tier (WGS84 geometry the engine projects and drapes itself) and a screen-space tier (decorations, handles and previews in screen units, already projected and clipped). Each item carries a role (`graphic`, `label`, `symbol`, `handle`) and a `PickRef`.

**Versioning and caching.** The crate exposes `RENDERER_VERSION`, recorded in every `Construction` and `RenderPlan` for reproduction and diagnosis. Rather than enumerating camera parameters, the host supplies opaque revisions that change whenever their inputs change:

- `view_revision`: camera position and orientation, viewport, projection, DPI;
- `surface_revision`: terrain and elevation data.

A `Construction` is keyed by (definition ID, definition revision, `RENDERER_VERSION`, configuration). A `RenderPlan` adds `view_revision`, `surface_revision`, font-metrics identity and style. Anything not in the key must not affect the output.

## Projection, clipping and fonts

- Geographic quantities (area extent, corridor width, range-fan radius) use metres on the ellipsoid and degrees. Screen quantities (stroke width, font size, tooth spacing, arrowhead size where the standard says so) use screen units. Never convert screen pixels into degrees and persist them.
- Tests check that zoom keeps geographic widths fixed on the ground and screen widths fixed in pixels.
- `Projection` is an injected trait. Per point it maps geo + altitude to a screen position or reports the point hidden (behind the horizon, behind the camera, outside the view volume), and maps screen to geo. It also states the screen-space tolerance used for densification.
- Per-point visibility is not enough for lines and areas. The core densifies geodesic segments until each projected sub-segment is within the tolerance, then clips the screen-space tier at the visibility boundary, locating crossings by bisection on the geodesic, so no segment is drawn across the horizon. The geographic tier is densified the same way but clipped by the engine.
- Altitudes carry an explicit datum (above ground, above mean sea level, above ellipsoid). Graphics are clamped to the ground: the `Projection` trait projects the ground at a point, on the host's terrain where it has terrain. Control points with altitudes are refused at construction, with a typed error, until heights and datums are carried through construction and projection.
- Flat Web Mercator is never assumed: tilted views, globe and vertical perspective all go through the same trait.
- Edges between control points are geodesics on WGS84. The primary consumer draws a 3D globe with terrain, where the geodesic is the line that looks straight and is the same in every projection; a line straight on a flat Mercator map is a rhumb line, which looks bent on the globe. The two agree to within 5 m over 10 km at up to 70° latitude, but differ by about 60 m over 50 km and 230 m over 100 km at 50°N (east–west edges, worst case), so long edges bow on flat maps. A rhumb-line option, if operators on flat maps need one, is a declared per-symbol or per-config choice, never a silent change of the default.
- Font measurement is an injected `FontMetrics` trait. Tests use a deterministic built-in implementation. Engine-specific width rules live in the adapter's implementation.
- Antimeridian, polar and very large extents are first-class inputs, not edge cases.

## Editing

- The core emits semantic handles (endpoint, vertex, width, radius, bearing, offset, …), each with a stable handle ID tied to the definition.
- Editing is pure functions: `(definition, handle, new position, view) → Result<definition>`, including vertex insert/delete where the symbol allows it. UI code only collects input and selection state.
- Edits honour the symbol's constraints (e.g. ratio locks, minimum point counts). An invalid edit returns an error and leaves the definition unchanged.
- **Cancel** of an uncommitted preview leaves the original definition exactly unchanged. **Undo** of a committed edit restores the earlier content as a new edit with a new revision, so history only moves forward.
- The revision field is a change counter advanced with `wrapping_add(1)`. It is not guaranteed to increase forever and it does not identify commands. Idempotency and conflict detection use the host's command/event IDs (Sokoly's journal event IDs and `supersedes`). The host owns that policy.

## Shared construction primitives

Port the algorithms, not Mission Command's class hierarchy. Extract a small set of shared primitives, then compose symbol families from them:

| Primitive | Primary reference location (mil-sym-java) |
|---|---|
| Polyline sampling, offsets, parallel lines, arrowheads | `JavaLineArray/lineutility.java`, `arraysupport.java` |
| Axes of advance, corridors, channels | `JavaLineArray/Channels.java` |
| Repeated decorations (teeth, FLOT scallops, wire, dashes) | `JavaLineArray/flot.java`, `arraysupport.java` |
| Mission tasks (block, bypass, fix, canalize, …) | `JavaLineArray/DISMSupport.java` |
| Geodesic arcs, circles, range fans | `JavaTacticalRenderer/mdlGeodesic.java` (reimplement; see licensing) |
| Label anchoring and modifier placement | `JavaTacticalRenderer/Modifier2.java` |
| Control-point limits per draw rule | `renderer/utilities/MSInfo.getMinMaxPointsFromDrawRule` |
| METOC lines and areas | `JavaTacticalRenderer/clsMETOC.java` |

Mission Command's 90 `DrawRules` and 19 `MODrawRules` map to Rust enums. Group symbols by draw rule when planning work.

## Public API (sketch)

```rust
let catalog = Catalog::new(StandardVersion::Mil2525Dch1);
let stored = PersistedGraphic::from_json(bytes)?;      // never loses content
let def = catalog.decode(&stored)?;                     // or Unsupported(stored)
let construction = catalog.construct(&def)?;           // geographic, cacheable
let plan = construction.render(&view, &projection, &font_metrics)?;
for item in plan.items() { /* geometry, labels, symbol placements, handles, pick refs */ }
let edited = catalog.apply_edit(&def, handle_id, new_position, &view)?;
```

Names will change once real integration shows a cleaner shape. Errors are `thiserror` enums with context (symbol code, version, point index, counts).

A deterministic SVG export of a `RenderPlan` under a fixed test projection serves golden tests, examples and debugging. It is not the internal representation.

## References, oracle and licensing

| Reference | Use | License | Pin |
|---|---|---|---|
| [mil-sym-java](https://github.com/missioncommand/mil-sym-java) | Primary algorithm reference and diagnostic oracle | Apache-2.0 | `v2.9.7`, commit `6634a2e916fb3513953ecc8d90a93371f05e6f4a` |
| [mil-sym-ts](https://github.com/missioncommand/mil-sym-ts) | Line-for-line TS port of the Java code; cross-check only | Apache-2.0 | `v2.10.7`, commit `37e6af70c5b33f8831647b19432e75f2b5ad7f57` |
| [zaes-code/tactical-graphics](https://github.com/zaes-code/tactical-graphics) | Architecture reference: control points → geometry/labels/handles, adapter boundary, pure edit functions | MIT | commit `c6019bc53dc0813147ee6e8492b4410395cec722` |
| MIL-STD-2525D/E, APP-6D/E official texts | Final authority on meaning and appearance | Official publications | Cite by document and section |

- Record pins and file-level provenance (which upstream file, at which commit, feeds which generated file or ported function) in `tools/oracle/pin.json` and `UPSTREAM.md`. `pin.json` is authoritative; update this table when it changes.
- Ported Apache-2.0 / MIT code keeps its copyright and license notice in `NOTICE` and in the license files (`LICENSE-APACHE-mil-sym`, `LICENSE-MIT-zaes`), and is noted in `UPSTREAM.md`.
- **Do not port or copy:**
  - mil-sym-java `web/json/utilities/{JSONObject,JSONArray,JSONTokener,HTTPTokener}.java` (JSON license).
  - mil-sym-ts `graphics2d/Line2D.ts` (GPL-2.0 with Classpath exception).
  - The CC-BY-3.0-marked intersection routine in `mdlGeodesic` (around line 327). Reimplement it from the math or from an MIT/Apache source.
  - Bundled single-point icon SVGs (`svgd`/`svge`/…). Single-point symbols are `milsymbol`'s job.
  - Standard texts, figures or text dumps of them (for example Zaes's `docs/FM_1-02.2.txt`) until their redistribution status is confirmed. Cite official URLs and section numbers instead.

### Oracle tooling

- JVM-based, dev-only, under `tools/oracle/`. It fetches the pinned mil-sym-java, runs headless (`-Djava.awt.headless=true`), and emits canonical JSON Lines records.
- Upstream's `src/test/java/armyc2/c5isr/RendererTests.java` is a template for calling `WebRenderer.RenderSymbol`.
- Mission Command keeps static mutable state (`RendererSettings`, `Channels`, `arraysupport`, …). Pin every renderer setting (DPI 96, font family/size/weight) and run cases in an isolated, reset configuration.
- Capture both pixel-space output (`RenderMultiPointAsMilStdSymbol` → symbol and modifier `ShapeInfo`, before back-projection) and GeoJSON output, plus the scale, bbox and settings used.
- Normal `cargo build` / `cargo test` never needs Java or Node. Checked-in fixtures run in ordinary CI; a separate oracle job regenerates and diffs them.

## Comparison policy

This differs from `milsymbol`, which reproduces its upstream bit for bit, bugs included. Mission Command computes in pixel space on a local equirectangular projection with platform-dependent font metrics, so bit-exact parity is neither achievable nor desirable.

- The standard text and figures decide what is correct. When the reference and the standard disagree, follow the standard, and record the divergence and its evidence in `UPSTREAM.md`.
- **Normalize before comparing.** Both sides are brought to one canonical representation: merge pieces split only for drawing or clipping, merge collinear segments, remove duplicated closing points, fix ring start and orientation, and resample to a common spacing. Raw shape counts are never a correctness criterion on their own.
- Then compare:
  1. semantics: which parts exist (boundary, decoration, arrowhead, label), open/closed, fill vs stroke;
  2. label set: which modifiers appear and their text;
  3. geometry within a stated tolerance per symbol family (e.g. Hausdorff distance in pixel space at a pinned scale);
  4. label anchors only with pinned fonts, otherwise excluded.
- Raster comparison is a diagnostic aid with explicit tolerances, never the acceptance criterion.
- Never accept a mismatch because it "looks similar". Each accepted difference is listed with its reason.

## Test and check targets

Tests ship with the code they cover. A symbol is not delivered without its tests.

| Area | Target |
|---|---|
| Standard coverage | Every declared (version, symbol) has control-point tests, modifier tests, a reviewed golden SVG, an oracle fixture and a standard reference (APP-6(D): oracle agreement only, see "Standards and coverage"). Undeclared combinations fail explicitly. |
| Reference comparison | The same inputs are compared with the pinned oracle under the comparison policy, plus human review against the standard's figures. |
| Cross-platform | Behaviour tests **execute** on native and in browser WASM (`wasm-bindgen-test`), not only compile. Semantics, item order, labels and handles match; coordinates match within tolerance. |
| Geometric robustness | Coincident points, zero-length and very short segments, sharp turns, reversed order, antimeridian crossing, poles, continent-scale extents, segments crossing the horizon. No NaN, no infinite loop, no runaway allocation. Property tests and fuzzing. |
| Budgets | Oversized inputs (control points, text, modifier arrays) and inputs that would generate too many vertices, decorations or labels fail with typed errors under stated limits. |
| Map display | In Sokoly-App: zoom, rotation, pitch, terrain and horizon behave correctly; geographic widths stay fixed on the ground and screen widths in pixels. |
| Editing | In Sokoly-App, through the real input path: pick, drag, cancel, undo, save, reload and edit again. |
| Persistence | Unknown fields keep their JSON content; unknown schema and standard versions keep their exact bytes, through decode → host journal → write. |
| Engine mapping | Plans convert to GeoJSON (antimeridian-split, enumerated dash patterns) and to a screen-space tier; every rendered pick ID maps back to its `PickRef`, and stale IDs are rejected. |
| Performance | Benchmarks for construction, label layout and plan generation at 100 / 1,000 / 10,000 graphics on fixed hardware, with control-point and cache-byte limits. |
| Boundaries | The library builds and tests offline without `milsymbol`, a map engine, a window or a GPU. No global mutable state, hidden I/O or platform-specific dependencies. |

## Implementation phases

Keep each PR to one issue.

1. **Foundations.** Workspace, lint table, CI gates (including WASM test execution, MSRV and feature combinations), `pin.json` with verified commits, file-level provenance, the standard references needed for the six graphics, oracle tooling, fixture format.
2. **Core types.** Standard versions, symbol identity, `PersistedGraphic`, `GraphicDefinition`, typed modifiers, errors, budgets, `Projection` and `FontMetrics`, and the catalog generated from `msd.txt` / `mse.txt` (data only, no support claim).
3. **First milestone: six representative graphics plus a minimal real map integration.** Each graphic is delivered complete — rules, construction, both plan tiers, handles and edits, oracle fixture, edge-case tests, golden SVG — and verified in this repository first. Then a minimal Sokoly-App adapter, with the fork and host changes it needs, verifies display, picking and editing on native and in the browser.

   | Graphic | Entity | Draw rule (2525D ch1 / 2525E ch1) | Exercises |
   |---|---|---|---|
   | Phase Line | 140300 | Line2 / Line1 | open polyline, end labels |
   | Named Area of Interest | 120200 | Area1 | closed area, interior label |
   | Main Attack | 151403 | Axis2 / Axis1 | axis of advance, arrowhead, 3–50 points |
   | Air Corridor | 170100 | Corridor1 | geographic width (`AM`), multi-field labels |
   | Weapon/Sensor Range Fan, Sector | 242200 | Arc1 | geodesic arcs, `AM`/`AN` arrays |
   | Bypass Easy | 270601 | Point12 | fixed 3-point mission task with decorations |

   Draw rules are per edition; table and page references are in `tools/oracle/pin.json`.

4. **Stabilize.** Review the public interfaces against what the integration showed, change them where the evidence says so, and only then treat them as stable.
5. **Extend** coverage family by family, reusing the shared primitives. Each family is accepted with its tests before the next starts.
6. **Host hooks** for history replay, synchronization and MCP query/edit (e.g. a stable query API over definitions and plans). The journal, sync and MCP servers themselves live in the host.

### First-milestone acceptance

- Behaviour tests execute on native and in browser WASM.
- Geographic widths, screen-space decorations, horizon and antimeridian behaviour are correct in the real map.
- Pick, drag, cancel, undo, save and reload go through the real application path.
- Unknown-field and unknown-version preservation tests pass end to end through the host journal.
- Input and generated-output budget tests pass.
- The library builds on Rust 1.85 (MSRV check), passes the full gate on current stable, and passes an explicit feature-combination check.

## Generated data and provenance

- Symbol catalog data (entity hierarchy, 6-digit codes, versions, geometry, draw rule, modifiers) is generated mechanically from the pinned upstream data files into `src/generated/` by `tools/codegen` + `cargo xtask`. It is never edited by hand.
- `UPSTREAM.md` records, for every generated file, the upstream file, revision and the command that regenerates it. The oracle CI job checks that regeneration produces no diff.
- Construction algorithms are hand-written Rust. They are not generated.

## Dependencies

- Geodesics: `geographiclib-rs` (MIT, pure Rust, depends on `libm`). It agrees with Karney's reference GeographicLib, runs on `wasm32-unknown-unknown` (Node runner; headless-browser execution is a phase-1 CI job), builds on Rust 1.85, and is already in both consumers' lockfiles. It requires `std`. Porting Karney's algorithms is a separate, explicitly approved project, never a side effect of pursuing `no_std`.
- Geometry: the library's own `GeoPoint` (finite, latitude in range, longitude normalized) and planar helpers; `geo` or `geo-types` only where an algorithm is worth its weight.
- `lyon` tessellation only if a consumer cannot tessellate the plan itself.
- Persistence: `serde` and `serde_json` (with `raw_value` for the envelope and `float_roundtrip` so stored coordinates round-trip exactly) are required dependencies, because persistence is part of the core contract.
- Every new dependency must be pure Rust, offline, `wasm32`-clean, AGPL-compatible, and checked for its effect on the consumers' lockfiles.

## Quality gates

- Edition 2024, current stable Rust, `rust-version = "1.85"` (matching `milsymbol`, below the consumers' 1.97.1).
- Use `milsymbol`'s workspace lint table: `forbid` `unsafe_code`, `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `exit`, `mem_forget`, `get_unwrap`, `indexing_slicing`, `string_slice`; `deny` `let_underscore_must_use`, `let_underscore_future`, `await_holding_lock`, `too_many_lines`; `missing_docs` enforced in the doc gate. Ban `anyhow::Error` in library code via `disallowed_types`.
- `clippy.toml` sets `too-many-lines-threshold = 80` and the test exemptions `allow-dbg-in-tests`, `allow-expect-in-tests`, `allow-indexing-slicing-in-tests`, `allow-panic-in-tests`, `allow-print-in-tests`, `allow-unwrap-in-tests` (all `true`). No `#[allow]` overrides of forbidden lints in tests.
- No clap derives or `#[tokio::main]` / `#[tokio::test]` in tooling (they emit forbidden-lint overrides).
- Limits: 500 lines per hand-written `.rs` file (`src/generated/` exempt), 80 per function, 30 fields per struct, 30 variants per enum (generated symbol enums exempt). `lib.rs` starts with a `//!` doc comment, stays under 100 lines, and only declares modules and re-exports. Use `foo.rs` + `foo/`, never `mod.rs` or `utils`/`helpers`/`common` modules.
- Unit tests in `src/**/tests.rs`; `tests/` uses only the public API.
- Local gate order, wrapped in `tools/ci/check.sh` and run by CI: `cargo fmt --check` → `cargo clippy --all-targets -- -D warnings` → `cargo test --all-targets` → `cargo doc` with `RUSTDOCFLAGS="-D missing_docs -D rustdoc::broken_intra_doc_links"` → `cargo build --release`. Separate jobs: WASM behaviour tests in a headless browser, MSRV build, `cargo hack` feature combinations, file-size limits, dependency-boundary guard.
- Comments explain why, not history. No change narratives in code or durable docs.
- Durable docs are limited to `AGENTS.md`, `README.md`, `UPSTREAM.md`, `NOTICE`, `LICENSE` and third-party license files, and `CHANGELOG.md` once releases begin. No per-module READMEs, status reports or plan documents in the repository.

## Acceptance criteria

The library is complete for a declared scope only when:

- every declared (version, symbol) passes its golden, oracle-comparison, edge-case, budget, editing and persistence tests, executed on native and in browser WASM;
- it displays, picks and edits correctly in Sokoly-App;
- every undeclared combination fails with a typed error;
- each remaining difference from the reference is individually justified against the standard in `UPSTREAM.md`;
- the library builds and runs offline with no JVM, JavaScript, map engine, window or GPU dependency;
- a native example turns definitions into a `RenderPlan` and an SVG without any external runtime.
