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
</tr></table>

## Usage

```rust
use milgraphics::render::{FixedAdvanceMetrics, Font, LocalEquirectangular};
use milgraphics::{Budget, Config, ControlPoint, GeoPoint, GraphicDefinition, GraphicId};
use milgraphics::{SymbolId, View, construct, render};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// A MIL-STD-2525D change 1 phase line, "PL ALPHA".
let points = vec![
    ControlPoint::ground(GeoPoint::new(20.0, 50.0)?),
    ControlPoint::ground(GeoPoint::new(20.1, 50.02)?),
];
let symbol = SymbolId::parse("11032500001403000000")?;
let mut def = GraphicDefinition::new(GraphicId::new("pl-1")?, symbol, points);
def.modifiers.designation = Some("ALPHA".into());

// Geographic construction, then a plan for one view. A map host passes its
// own projection and font metrics; this fixed frame suits tests and SVG.
let construction = construct(&def, &Config::default())?;
let frame = LocalEquirectangular::new(19.95, 50.07, 50_000.0, 96.0);
let view = View::new(0, 0);
let plan = render(&construction, &view, &frame, &FixedAdvanceMetrics::default(), &Budget::default())?;

let svg = milgraphics::svg::to_svg(&plan, 1100.0, 900.0);
# assert!(svg.contains("PL ALPHA"));
# Ok(())
# }
```

`cargo run --example render_svg` renders a few graphics to SVG; the images
above are three of them. `milgraphics::support::all()` lists every supported
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
