//! The public types hosts store, compare, log and share between threads keep
//! the traits that make that possible.

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// Clippy does not recognise `wasm_bindgen_test` functions as tests, so the
// test exemptions in clippy.toml reach them only through a `cfg(test)` module.
#[cfg(test)]
mod traits {
    use std::fmt::Debug;

    use milgraphics::render::{GeoItem, Label, ScreenItem, SymbolPlacement};
    use milgraphics::{
        Construction, Edit, EditContext, Edited, GraphicDefinition, Modifiers, PersistedGraphic,
        PickRef, RenderPlan, SymbolSpec, View,
    };

    fn shared<T: Clone + Debug + Send + Sync + 'static>() {}
    fn comparable<T: PartialEq>() {}

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn hosts_can_cache_compare_and_share_outputs() {
        shared::<RenderPlan>();
        shared::<Construction>();
        shared::<GraphicDefinition>();
        shared::<PersistedGraphic>();
        shared::<SymbolSpec>();
        shared::<Modifiers>();
        shared::<Edit>();
        shared::<Edited>();
        shared::<EditContext>();
        shared::<View>();
        shared::<GeoItem>();
        shared::<ScreenItem>();
        shared::<Label>();
        shared::<SymbolPlacement>();
        shared::<PickRef>();
        comparable::<RenderPlan>();
        comparable::<Construction>();
        comparable::<GraphicDefinition>();
        comparable::<PersistedGraphic>();
        comparable::<SymbolSpec>();
        comparable::<Modifiers>();
        comparable::<Edit>();
        comparable::<Edited>();
        comparable::<View>();
    }
}
