# milgraphics

Multi-point military tactical graphics per **MIL-STD-2525** and **APP-6**, in
native Rust: control measures such as phase lines, areas, axes of advance,
corridors and range fans.

`milgraphics` turns a graphic definition (symbol identity, geographic control
points, typed modifiers) into map-engine-neutral output — geometry, labels,
symbol placements, edit handles and pick references — that a map adapter
draws. It is the sibling of [milsymbol](https://github.com/luofang34/milsymbol),
which renders single-point symbols; applications use both, and neither
depends on the other.

The same code runs natively and in the browser through WebAssembly, with no
JavaScript, JVM or map engine at runtime.

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
