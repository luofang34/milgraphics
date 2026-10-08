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

APP-6(D) and APP-6(E) need an ASSIST account and are not yet referenced, so no
APP-6 symbol can be declared supported.

Draw rules are per edition. Phase Line is Line2 in 2525D change 1 and Line1 in
2525E change 1; Main Attack is Axis2 and Axis1 respectively (2525E change 1
marks Axis2 "Disused").

## Divergences under review

| Symbol | Standard | mil-sym-java | Status |
|---|---|---|---|
| Main Attack 151403, 2525E change 1 | TABLE L-X: draw rule Axis1 | `mse.txt`: Axis2 | Declared in `src/support.rs` (`catalog_divergence`); geometry is the same for both editions and matches the oracle |

## Accepted differences from the oracle

Each is bounded by a test in `tests/oracle_compare.rs`.

| Graphic | Difference | Reason | Bound |
|---|---|---|---|
| All | Edges are WGS84 geodesics; mil-sym draws straight lines in its local pixel frame | Geographic correctness across projections; at tactical scale the two agree | Geometry within 0.5–1.5 px |
| Main Attack | Body half-width is exactly half the width point's offset; mil-sym truncates to whole pixels | Ground-sized geometry must not depend on the display scale | Geometry within 1.5 px |
| Main Attack | The designation is anchored at the geodesic midpoint of its control-point segment; mil-sym uses a midpoint of its internally adjusted pixel path | Label position must follow the control points, not renderer internals | Up to 12 px along the text, 2.5 px across |
| Main Attack | `W`/`W1` are not drawn and are refused | Their text format on axes is not yet verified against the standard | Typed error |
| Air Corridor | Empty `DTG Start:`/`DTG End:` lines are omitted; mil-sym prints them with no value | The standard's information block shows fields that have values | Label sets compared without empty fields |
| Air Corridor | Control-point circles have 72 vertices and the radius is `AM`/2 on the ground; mil-sym uses 25-gons sized from a pixel scale measured northward at point 1 | Ground-sized geometry | Geometry within 1.5 px |
| Range Fan, Sector | Ranges are measured on the WGS84 ellipsoid; mil-sym uses a sphere of radius 6,378,137 m | Correct ground distances | Geometry within 1.5 px |
| Named Area of Interest | The label sits at the centre of the longitude/latitude bounds; mil-sym walks a spherical "minimum bounding rectangle" | Equivalent within a metre at tactical scale | Label within 1.5 px |

## Generated data

`src/generated/` is mechanically extracted data from the Apache-2.0
mil-sym-java at the pinned commit; no upstream code is incorporated. Its
license text is `LICENSE-APACHE-mil-sym` and the attribution is in `NOTICE`.
Regenerate after `tools/oracle/fetch-upstream.sh` with `cargo xtask catalog`;
the CI `oracle` job fails if the result differs from the checked-in files.

| Generated file | Upstream source files | Command |
|---|---|---|
| `src/generated/catalog.rs` | `src/main/resources/data/msd.txt`, `mse.txt` (parsed as `MSLookup.java` does) | `cargo xtask catalog` |
| `src/generated/draw_rule.rs` | `DrawRules.java`, `MODrawRules.java` | `cargo xtask catalog` |

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
differing case and field, if the result differs from the checked-in fixtures.
Ordinary CI uses the checked-in fixtures and needs no JDK.
