//! The checked-in oracle fixtures match their case list and the record format
//! that later comparison tests rely on.

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// Clippy does not recognise `wasm_bindgen_test` functions as tests, so the
// test exemptions in clippy.toml reach them only through a `cfg(test)` module.
#[cfg(test)]
mod fixtures {
    use serde_json::Value;

    const CASES: &str = include_str!("../tools/oracle/cases/six.tsv");
    const RECORDS: &str = include_str!("fixtures/oracle/six.jsonl");

    /// The fields of one case line that the oracle echoes back.
    struct Case {
        id: &'static str,
        symbol: &'static str,
        points: &'static str,
        bbox: &'static str,
    }

    fn cases() -> Vec<Case> {
        CASES
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .map(|l| {
                let f: Vec<&str> = l.split('\t').collect();
                Case {
                    id: f[0],
                    symbol: f[1],
                    points: f[2],
                    bbox: f[4],
                }
            })
            .collect()
    }

    fn records() -> Vec<Value> {
        RECORDS
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn one_record_per_case_in_order() {
        let cases = cases();
        let records = records();
        assert_eq!(
            cases.len(),
            records.len(),
            "regenerate with tools/oracle/oracle.sh"
        );
        for (case, record) in cases.iter().zip(&records) {
            assert_eq!(record["case"], case.id);
            assert_eq!(record["symbol"], case.symbol);
            assert_eq!(record["control_points"], case.points);
            assert_eq!(record["bbox"], case.bbox);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn rendered_records_carry_shapes_and_valid_geojson() {
        for record in records() {
            let geojson: Value = serde_json::from_str(record["geojson"].as_str().unwrap()).unwrap();
            if record["can_render"] != "true" {
                assert_eq!(geojson["type"], "error", "{}", record["case"]);
                continue;
            }
            let shapes = record["symbol_shapes"].as_array().unwrap();
            assert!(!shapes.is_empty(), "{}", record["case"]);
            for shape in shapes {
                for line in shape["polylines"].as_array().unwrap() {
                    for point in line.as_array().unwrap() {
                        let xy = point.as_array().unwrap();
                        assert!(
                            xy.len() == 2 && xy.iter().all(Value::is_number),
                            "{}",
                            record["case"]
                        );
                    }
                }
            }
            assert_eq!(geojson["type"], "FeatureCollection", "{}", record["case"]);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn every_record_was_rendered_with_the_pinned_font() {
        for record in records() {
            assert_eq!(
                record["font_probe"]["family"], "PT Sans",
                "{}",
                record["case"]
            );
        }
    }
}
