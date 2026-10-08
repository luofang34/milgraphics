# Upstream references and provenance

`tools/oracle/pin.json` is the machine-readable record of everything below:
reference repositories and commits, the sha256 of every upstream file the
project derives from or calls, and the oracle's runtime jars and font.
`tools/oracle/fetch-upstream.sh` refuses to proceed unless all of them match.

## References

| Reference | Role | License | Pin |
|---|---|---|---|
| [mil-sym-java](https://github.com/missioncommand/mil-sym-java) | Primary algorithm reference and diagnostic oracle | Apache-2.0 | `v2.9.7`, `6634a2e916fb3513953ecc8d90a93371f05e6f4a` |
| [mil-sym-ts](https://github.com/missioncommand/mil-sym-ts) | Cross-check only | Apache-2.0 | `v2.10.7`, `37e6af70c5b33f8831647b19432e75f2b5ad7f57` |
| [zaes-code/tactical-graphics](https://github.com/zaes-code/tactical-graphics) | Architecture reference | MIT | `c6019bc53dc0813147ee6e8492b4410395cec722` |

The standards decide correctness; a reference's output is evidence, not proof
(see the comparison policy in `AGENTS.md`).

## Standards

The standards are identified in `pin.json` under `standards`: MIL-STD-2525D,
2525D change 1, 2525E and 2525E change 1, from the DLA ASSIST QuickSearch
listing, all Distribution A. ASSIST stamps every page with the download time,
so a file hash identifies one copy only; documents are identified by revision,
cover date and page count. Each first-milestone graphic has its table and PDF
page in 2525D change 1 and 2525E change 1, and the draw rule printed there.

APP-6(D) needs an ASSIST account and is not referenced. Its symbols (version
code 10) are declared on agreement with the oracle alone: every multipoint
graphic upstream draws for them is declared when the ported renderer matches
it, with upstream's catalog draw rule, and `SymbolSpec::reference` is `None`.

APP-6 symbols are drawn by the ported renderer throughout, including the six
graphics that 2525D/E change 1 draw with milgraphics' own families.

Draw rules are per edition. Phase Line is Line2 in 2525D change 1 and Line1 in
2525E change 1; Main Attack is Axis2 and Axis1 respectively (2525E change 1
marks Axis2 "Disused").

## Graphics upstream draws that are not declared

Upstream's catalog lists some graphics the standard edition does not define
as multipoint graphics; they are not declared, so they are refused like any
undeclared symbol (`tests/fixtures/oracle/unimplemented.txt`):

| Graphic | Edition | Standard |
|---|---|---|
| Line of Contact 25 140200 | 2525D change 1, 2525E change 1 | No row: reserved in 2525D, a Combat Support code in 2525E |
| Wind Plot 45 140200 | 2525E change 1 | No row (2525D change 1 defines it in TABLE I-II) |

Some version 16 codes have no line type in upstream's `getCMLineType`
(Bridgehead, Mobility Corridor, Avenue of Approach 152300, Restricted Terrain,
Severely Restricted Terrain, Navigational Rhumb Line, Rectangular Target 240804,
AMA, ARA, Zone of Fire, the 242700 areas, Recover, Human Terrain). Upstream
draws only their control points as a line, which is not the graphic, so they
are not declared either.

## Divergences under review

| Symbol | Standard | mil-sym-java | Status |
|---|---|---|---|
| Main Attack 151403, 2525E change 1 | TABLE L-X: draw rule Axis1 | `mse.txt`: Axis2 | The standard's rule is followed; geometry is the same for both editions and matches the oracle |
| Trip Wire 290500, 2525D change 1 | Line15 | `msd.txt`: Line1 | Drawn as upstream draws it (matches the oracle); point limits follow upstream's rule |
| Rectangular Target 240802, 2525E change 1 | Rectangular1 | `mse.txt`: Rectangular2 | As above |
| Ferry 290700, 2525E change 1 | Line14 | `mse.txt`: Line18 | As above |
| Withdraw 342400 and Withdraw Under Pressure 342500, 2525E change 1 | Line24 | `mse.txt`: Line14 | As above |

Each is listed in `src/support/tests.rs` (`DIVERGENCES`), which fails if a
declared symbol's printed and catalog rules differ without being listed.

## Accepted differences from the oracle

Each is bounded by a test in `tests/oracle_compare.rs`.

| Graphic | Difference | Reason | Bound |
|---|---|---|---|
| All | Edges are WGS84 geodesics; mil-sym draws straight lines in its local pixel frame | Geographic correctness across projections; at tactical scale the two agree | Geometry within 0.5–1.5 px |
| Main Attack | Body half-width is exactly half the width point's offset; mil-sym truncates to whole pixels | Ground-sized geometry must not depend on the display scale | Geometry within 1.5 px |
| Main Attack | The designation is anchored at the geodesic midpoint of its control-point segment; mil-sym uses a midpoint of its internally adjusted pixel path | Label position must follow the control points, not renderer internals | Up to 12 px along the text, 2.5 px across |
| Air Corridor | Empty `DTG Start:`/`DTG End:` lines are omitted; mil-sym prints them with no value | The standard's information block shows fields that have values | Label sets compared without empty fields |
| Air Corridor | The information block is stacked outside the corridor edge of the first segment that is uppermost on screen, and "AC T" is placed inside every segment; mil-sym stacks the block across the middle segment, where it overlaps the corridor edge, with one "AC T" | 2525E change 1: the box goes "between points 1 and 2 in such a way it does not obscure the symbol", and the field inside appears "within each segment" | Label texts compared; positions reviewed in the golden SVGs |
| Air Corridor | Control-point circles have 72 vertices and the radius is `AM`/2 on the ground; mil-sym uses 25-gons sized from a pixel scale measured northward at point 1 | Ground-sized geometry | Geometry within 1.5 px |
| Range Fan, Sector | Ranges are measured on the WGS84 ellipsoid; mil-sym uses a sphere of radius 6,378,137 m | Correct ground distances | Geometry within 1.5 px |
| Named Area of Interest | The label sits at the centre of the longitude/latitude bounds; mil-sym walks a spherical "minimum bounding rectangle" | Equivalent within a metre at tactical scale | Label within 1.5 px |

## Ported renderer

Every multipoint graphic other than the six above is drawn by `src/engine/`,
a port of mil-sym-java's multipoint renderer (Apache-2.0, pinned commit in
`pin.json`) that runs in pixel space. milgraphics feeds it control points
projected for the view and draws its output as the screen tier; shapes that
come out identical on the ground at two pixel sizes (`src/family/ported.rs`)
also form the geographic tier.

| Port | Upstream source (mil-sym-java `src/main/java/armyc2/c5isr/`) |
|---|---|
| `engine/lineutility` | `JavaLineArray/lineutility.java` |
| `engine/arraysupport` | `JavaLineArray/arraysupport.java`, `countsupport.java` |
| `engine/flot` | `JavaLineArray/flot.java` |
| `engine/dism` | `JavaLineArray/DISMSupport.java` |
| `engine/channels`, `engine/channel_utility`, `engine/partition` | `JavaLineArray/Channels.java`, `CChannelPoints2.java`, `JavaTacticalRenderer/clsChannelUtility.java`, `P1.java` |
| `engine/metoc` | `JavaTacticalRenderer/clsMETOC.java` |
| `engine/modifier` | `JavaTacticalRenderer/Modifier2.java` |
| `engine/tg`, `engine/tg_utility`, `engine/line_type`, `engine/tactical_lines` | `JavaTacticalRenderer/TGLight.java`, `clsUtility.java`, `RenderMultipoints/clsRenderer.java` (`getCMLineType`), `clsMETOC.java` (`getWeatherLinetype`), `JavaLineArray/TacticalLines.java`, `CELineArray.java` |
| `engine/build` | `RenderMultipoints/clsRenderer.java` (`createTGLightFromMilStdSymbol`), `web/render/MultiPointHandler.java` (`populateModifiers`) |
| `engine/intercept` | `web/render/WebRenderer.java` (`interceptAndAdjustCode`: APP-6 feint and dummy duplicates drawn as their originals with the feint/dummy indicator) |
| `engine/cpof`, `engine/render_utility`, `engine/pipeline` | `RenderMultipoints/clsUtilityCPOF.java`, `clsUtility.java`, `clsUtilityGE.java`, `clsClipPolygon2.java` (non-clipping parts), `clsRenderer2.java`, `clsRenderer.java` (`render_GE`) |

Clipping, SVG and raster output, and the geodesic densification of very long
edges are not ported. Where upstream converts between pixels and geographic
coordinates (`mdlGeodesic` through its converter), the port works in the
view's pixel frame with its ground scale (`meters_per_pixel`), which is
conformal over a graphic's extent; the `mdlGeodesic` code itself, including
its CC-BY-3.0 routine, is not ported.

`tests/oracle_catalog.rs` compares every case of `tools/oracle/cases/all.tsv`.
Accepted differences, each bounded by a per-case tolerance there:

| Graphic | Difference | Reason | Bound |
|---|---|---|---|
| Depth Area (46 120104) | Its two bands along the inside of the outline are drawn as mitred lines along their centres, as wide as the bands; upstream fills each band as a stroked area intersected with the polygon, a fill with a hole | The output's polygons are single rings; the drawn band is the same | 7.5 px (half the wider band) |

## Generated data

`src/generated/` is mechanically extracted data. The catalog and draw-rule
files come from the Apache-2.0 mil-sym-java at the pinned commit; no upstream
code is incorporated. Its
license text is `LICENSE-APACHE-mil-sym` and the attribution is in `NOTICE`.
Regenerate after `tools/oracle/fetch-upstream.sh` with `cargo xtask catalog`;
the CI `oracle` job fails if the result differs from the checked-in files.

| Generated file | Upstream source files | Command |
|---|---|---|
| `src/generated/catalog.rs` | `src/main/resources/data/msd.txt`, `mse.txt` (parsed as `MSLookup.java` does) | `cargo xtask catalog` |
| `src/generated/draw_rule.rs` | `DrawRules.java`, `MODrawRules.java` | `cargo xtask catalog` |

`src/generated/references.rs` holds, per symbol, the table, PDF page and
printed draw rule of each standard that defines it. It is generated by
`cargo xtask references` from `tools/oracle/references.json`, which
`tools/standards/references.py` extracts from the standard PDFs listed in
`pin.json` (only those three fields are recorded, never standard text).

All upstream paths are relative to the mil-sym-java root, and every file is
pinned by sha256 in `pin.json`. Where a later row of a data file repeats an
earlier row's symbol set, entity and version, the later row wins, as in
upstream's lookup table.

## Oracle

`tools/oracle/oracle.sh CASES.tsv > OUT.jsonl` renders tactical-graphic cases
with the pinned mil-sym-java. It needs git, Python 3 and a JDK (21 or later);
it never runs as part of `cargo build` or `cargo test`.

- The harness `tools/oracle/java/Oracle.java` is compiled with the upstream
  sources against the pinned `geodesy` and `jsvg` jars. Upstream's Gradle
  build is not used.
- mil-sym keeps renderer settings in static state, so every case runs in its
  own JVM, and the harness pins every setting it relies on: device DPI 96, no
  text background, label and multipoint label font PT Sans Bold 12.
- The font is the pinned `PT_Sans-Web-Bold.ttf` (OFL-1.1), registered with
  AWT rather than taken from the operating system. It is byte-identical to the
  face Sokoly-App ships. Each record carries a `font_probe` (family, advance of
  a fixed string, ascent, descent), so metric differences between platforms
  show up as a named field rather than as unexplained label drift.
- Upstream logs through standard output; the harness reserves standard output
  for records and sends upstream's output to standard error.

### Case files

`tools/oracle/cases/*.tsv`, one case per line, tab-separated; `#` starts a
comment line:

| Column | Content |
|---|---|
| 1 | case id |
| 2 | 20-digit symbol ID (version, context, standard identity, symbol set, status, HQ/TF/dummy, amplifier, entity, modifier 1, modifier 2) |
| 3 | control points, `lon,lat` separated by spaces |
| 4 | map scale denominator |
| 5 | bounding box `west,south,east,north` |
| 6… | modifiers as `KEY=VALUE`, keys being mil-sym `Modifiers` constants; multi-valued modifiers (`AM_DISTANCE`, `AN_AZIMUTH`, `X_ALTITUDE_DEPTH`) are comma-separated |

### Fixture records

`tests/fixtures/oracle/<cases>.jsonl` holds one JSON object per case, in case
order:

| Field | Content |
|---|---|
| `case`, `symbol`, `control_points`, `scale`, `bbox`, `modifiers` | the case, echoed |
| `font_probe` | `{family, text, width, ascent, descent}` of the label font as AWT measured it |
| `can_render` | `"true"`, or upstream's refusal message |
| `symbol_shapes`, `modifier_shapes` | upstream `ShapeInfo` lists in pixel space before back-projection, each `{shape_type, line_color, fill_color, stroke_width, dash, polylines, text, position, angle, justify}`; colours are `#rrggbbaa`; `null` when nothing was rendered |
| `geojson` | upstream's GeoJSON output, as a string |

Numbers use Java's shortest round-trip decimal form, and integral values are
written without a fractional part. Regenerating twice produces identical bytes.

Regenerate and check:

```sh
tools/oracle/oracle.sh tools/oracle/cases/six.tsv > tests/fixtures/oracle/six.jsonl
python3 -I tools/oracle/diff-fixtures.py tests/fixtures/oracle/six.jsonl /path/to/regenerated.jsonl
```

The CI `oracle` job regenerates every case file on Linux and fails, naming each
differing case and field, if the oracle's output differs from the checked-in
fixtures. `font_probe` differences are reported as a notice only: Java measures
the same PT Sans file slightly differently on Linux and macOS (advance 123 vs
124 px, ascent 13 vs 11 px for the probe string), while every shape and label
of the current cases is identical on both.
Ordinary CI uses the checked-in fixtures and needs no JDK.
