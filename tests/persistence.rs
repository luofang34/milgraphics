//! Stored graphics survive decode, construction and edits: unknown fields
//! keep their JSON content, undecodable graphics keep their bytes.

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// Clippy does not recognise `wasm_bindgen_test` functions as tests, so the
// test exemptions in clippy.toml reach them only through a `cfg(test)` module.
#[cfg(test)]
mod persistence {
    use milgraphics::{
        Config, ConstructError, Edit, GeoPoint, HandleId, PersistError, PersistedGraphic,
        Unsupported, apply_edit, construct,
    };
    use serde_json::Value;

    const FROM_NEWER_VERSION: &str = r##"{"schema":1,"id":"pl-3","symbol":"15032500001403000000",
        "points":[{"lon":20.0,"lat":50.0,"accuracy_m":12},{"lon":20.1,"lat":50.02}],
        "modifiers":{"T":"ALPHA"},"style":{"line_color":"#112233","halo":true},
        "revision":41,"classification":{"level":"U"}}"##;

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn unknown_fields_survive_an_edit_cycle() {
        let stored = PersistedGraphic::from_json(FROM_NEWER_VERSION).unwrap();
        let def = stored.decode().unwrap();
        construct(&def, &Config::default()).unwrap();
        let to = GeoPoint::new(20.2, 50.05).unwrap();
        let edited = apply_edit(
            &def,
            &Edit::Move {
                handle: HandleId::Vertex(1),
                to,
            },
        )
        .unwrap();
        let written = PersistedGraphic::from_definition(&edited).unwrap();
        let before: Value = serde_json::from_str(FROM_NEWER_VERSION).unwrap();
        let after: Value = serde_json::from_str(written.as_json()).unwrap();
        for path in [
            "/classification",
            "/style/halo",
            "/style/line_color",
            "/points/0/accuracy_m",
        ] {
            assert_eq!(after.pointer(path), before.pointer(path), "{path}");
        }
        assert_eq!(after["revision"], 42);
        assert_eq!(written.decode().unwrap(), edited);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn undecodable_and_unsupported_graphics_keep_their_bytes() {
        let future = r#"{"schema":7,"id":"x","shape":"spline"}"#;
        let stored = PersistedGraphic::from_json(future).unwrap();
        assert!(matches!(
            stored.decode(),
            Err(PersistError::UnknownSchema { .. })
        ));
        assert_eq!(stored.as_json(), future);

        let other_edition =
            FROM_NEWER_VERSION.replace("15032500001403000000", "17032500001403000000");
        let stored = PersistedGraphic::from_json(&other_edition).unwrap();
        let def = stored.decode().unwrap();
        assert!(matches!(
            construct(&def, &Config::default()),
            Err(ConstructError::Unsupported(Unsupported::Standard {
                code: 17
            }))
        ));
        assert_eq!(stored.as_json(), other_edition);
    }
}
