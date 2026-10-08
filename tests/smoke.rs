//! Proves the behaviour-test harness runs on native and in a browser.

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn renderer_version_is_the_crate_version() {
    assert_eq!(milgraphics::RENDERER_VERSION, env!("CARGO_PKG_VERSION"));
}
