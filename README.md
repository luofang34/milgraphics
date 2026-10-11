# milgraphics

Multi-point military tactical graphics per **MIL-STD-2525**, in native Rust:
every multipoint control measure and METOC line and area of MIL-STD-2525D
change 1 and 2525E change 1 — phase lines, areas, axes of advance,
corridors, range fans, obstacles, mission tasks, fronts and more.

`milgraphics` turns a graphic definition (symbol identity, geographic control
points, typed modifiers) into map-engine-neutral output — geometry, labels,
symbol placements, edit handles and pick references — that a map adapter
draws. It is the sibling of [milsymbol](https://github.com/luofang34/milsymbol),
which renders single-point symbols; applications use both, and neither
depends on the other.

The same code runs natively and in the browser through WebAssembly, with no
JavaScript, JVM or map engine at runtime.

<table><tr>
<td align="center"><img src="https://github.com/luofang34/milgraphics/raw/main/docs/images/main-attack.svg" width="260" alt="Main attack"><br>Main attack</td>
<td align="center"><img src="https://github.com/luofang34/milgraphics/raw/main/docs/images/air-corridor.svg" width="260" alt="Air corridor"><br>Air corridor</td>
<td align="center"><img src="https://github.com/luofang34/milgraphics/raw/main/docs/images/range-fan-sector.svg" width="260" alt="Range fan, sector"><br>Range fan, sector</td>
</tr><tr>
<td align="center"><img src="https://github.com/luofang34/milgraphics/raw/main/docs/images/flot.svg" width="260" alt="Forward line of troops"><br>Forward line of troops</td>
<td align="center"><img src="https://github.com/luofang34/milgraphics/raw/main/docs/images/double-apron-fence.svg" width="260" alt="Double apron fence"><br>Double apron fence</td>
<td align="center"><img src="https://github.com/luofang34/milgraphics/raw/main/docs/images/cold-front.svg" width="260" alt="Cold front"><br>Cold front</td>
</tr></table>

## Usage

```rust
use milgraphics::render::{FixedAdvanceMetrics, Font, LocalEquirectangular};
use milgraphics::{Config, ControlPoint, GeoPoint, GraphicDefinition, GraphicId};
use milgraphics::{ModifierField, ModifierValue, SymbolId, View, construct, render};
use milgraphics::svg::{SvgOptions, to_svg};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// A MIL-STD-2525D change 1 phase line, "PL ALPHA".
let points = vec![
    ControlPoint::ground(GeoPoint::new(20.0, 50.0)?),
    ControlPoint::ground(GeoPoint::new(20.1, 50.02)?),
];
let symbol = SymbolId::parse("11032500001403000000")?;
let mut def = GraphicDefinition::new(GraphicId::new("pl-1")?, symbol, points);
def.modifiers.set(ModifierField::T, ModifierValue::Text("ALPHA".into()))?;

// Geographic construction, then a plan for one view. A map host passes its
// own projection and font metrics; this fixed frame suits tests and SVG.
let construction = construct(&def, &Config::default())?;
let frame = LocalEquirectangular::new(19.95, 50.07, 50_000.0, 96.0);
let view = View::new(0, 0);
let plan = render(&construction, &view, &frame, &FixedAdvanceMetrics::default())?;

let svg = to_svg(&plan, &SvgOptions::new(1100.0, 900.0));
# assert!(svg.contains("PL ALPHA"));
# Ok(())
# }
```

Edits are pure functions. A host turns the dragged handle's screen position
into a geographic one with its projection, applies the edit, and stores the
result; the GeoJSON goes to the map engine, one layer per dash pattern.

```rust
# use milgraphics::{Config, ControlPoint, GeoPoint, GraphicDefinition, GraphicId, SymbolId};
use milgraphics::{Edit, EditContext, HandleId, PersistedGraphic, apply_edit};
use milgraphics::geojson::to_geojson;
# use milgraphics::render::{FixedAdvanceMetrics, LocalEquirectangular};
# use milgraphics::{View, render};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let points = vec![
#     ControlPoint::ground(GeoPoint::new(20.0, 50.0)?),
#     ControlPoint::ground(GeoPoint::new(20.1, 50.02)?),
# ];
# let symbol = SymbolId::parse("11032500001403000000")?;
# let def = GraphicDefinition::new(GraphicId::new("pl-1")?, symbol, points);
// Drag the second control point. A refused edit returns an error and
// leaves `def` as it was.
let context = EditContext::new(Config::default());
let edit = Edit::Move { handle: HandleId::Vertex(1), to: GeoPoint::new(20.12, 50.05)? };
let edited = apply_edit(&def, &edit, &context)?;
assert_eq!(edited.definition.revision, def.revision.wrapping_add(1));

// The persisted form keeps fields it does not know, so newer data survives
// an older reader.
let stored = PersistedGraphic::from_definition(&edited.definition)?;
let reloaded = PersistedGraphic::from_json(stored.as_json())?.decode()?;
assert_eq!(reloaded, edited.definition);

# let frame = LocalEquirectangular::new(19.95, 50.07, 50_000.0, 96.0);
# let (view, metrics) = (View::new(0, 0), FixedAdvanceMetrics::default());
// The edit returns the new construction, ready to render.
let plan = render(&edited.construction, &view, &frame, &metrics)?;
let features = to_geojson(&plan);
# assert!(features.to_string().contains("FeatureCollection"));
# Ok(())
# }
```

`cargo run --example render_svg` renders the graphics above and a few more
to SVG. `milgraphics::support::all()` lists every supported
symbol with its points, amplifiers and standard reference, for building
palettes and amplifier forms.

## Building and testing

```sh
tools/ci/check.sh        # fmt, clippy, tests, docs, release build and repository checks
tools/ci/wasm-test.sh    # behaviour tests in a headless browser
```

`tools/ci/wasm-test.sh` needs `wasm-bindgen-cli` at the version pinned in
`Cargo.lock` and a WebDriver: `chromedriver`, `geckodriver`, or Safari's
`safaridriver` on macOS.

## License

AGPL-3.0-or-later. See `LICENSE` and `NOTICE`.
