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

    /// A definition that sets every modelled field, so every nested object
    /// exists in its JSON form.
    const EVERY_FIELD: &str = r##"{"schema":1,"id":"c-1","symbol":"11032500001701000000",
        "points":[{"lon":20.0,"lat":50.0,"altitude":{"metres":10.0,"datum":"msl"}},{"lon":20.1,"lat":50.03}],
        "modifiers":{"T":"AC1","T1":"x","H":"h","W":"w","W1":"w1","AM":[2000.0],"AN":[1.0,2.0],
            "X":[{"metres":1000.0,"datum":"msl"}]},
        "style":{"line_color":"#000000","fill_color":"#ffffff"},
        "validity":{"start":"2026-01-01T00:00:00Z","end":"2026-01-02T00:00:00Z"},"revision":3}"##;

    /// Adds `"zz_<path>": "<path>"` to every object, depth first.
    fn tag_every_object(v: &mut Value, path: &str, tagged: &mut Vec<String>) {
        match v {
            Value::Object(map) => {
                for (k, child) in map.iter_mut() {
                    tag_every_object(child, &format!("{path}/{k}"), tagged);
                }
                let key = format!("zz_{}", path.replace('/', "_"));
                map.insert(key, Value::from(path.to_owned()));
                tagged.push(path.to_owned());
            }
            Value::Array(items) => {
                for (i, child) in items.iter_mut().enumerate() {
                    tag_every_object(child, &format!("{path}/{i}"), tagged);
                }
            }
            _ => {}
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn unknown_fields_survive_in_every_nested_object() {
        let mut stored: Value = serde_json::from_str(EVERY_FIELD).unwrap();
        let mut tagged = Vec::new();
        tag_every_object(&mut stored, "", &mut tagged);
        assert!(tagged.len() >= 8, "every level is covered: {tagged:?}");
        let json = serde_json::to_string(&stored).unwrap();
        let def = PersistedGraphic::from_json(&json)
            .unwrap()
            .decode()
            .unwrap();
        // Edits refuse this graphic (a control point has an altitude, which
        // is stored but not yet drawn); edits on supported graphics are
        // covered by `unknown_fields_survive_an_edit_cycle`.
        let to = GeoPoint::new(20.2, 50.05).unwrap();
        let refused = apply_edit(
            &def,
            &Edit::Move {
                handle: HandleId::Vertex(1),
                to,
            },
        );
        assert!(refused.is_err());
        for written in [&def] {
            let out: Value = serde_json::from_str(
                PersistedGraphic::from_definition(written)
                    .unwrap()
                    .as_json(),
            )
            .unwrap();
            for path in &tagged {
                let key = format!("zz_{}", path.replace('/', "_"));
                let object = if path.is_empty() {
                    Some(&out)
                } else {
                    out.pointer(path)
                };
                assert_eq!(
                    object.and_then(|o| o.get(&key)),
                    Some(&Value::from(path.clone())),
                    "unknown field at {path:?} was lost"
                );
            }
        }
    }
}
