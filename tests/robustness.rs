//! Robustness of the public pipeline: arbitrary geometry and amplifiers must
//! give typed errors or finite output, never panics, and edits must stay pure.

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// Clippy does not recognise `wasm_bindgen_test` functions as tests, so the
// test exemptions in clippy.toml reach them only through `cfg(test)` modules.
#[cfg(test)]
#[path = "robustness/edge.rs"]
mod edge;
#[cfg(test)]
#[path = "robustness/fixtures.rs"]
mod fixtures;
#[cfg(test)]
#[path = "robustness/pan.rs"]
mod pan;
#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "robustness/props.rs"]
mod props;
