//! Aids for reviewing the oracle cases by eye and for timing them; each
//! runs only when its environment variable is set.

use super::*;

/// `ORACLE_TIMING=1 cargo test --release --test oracle_catalog timing -- --nocapture`
/// prints construction and render times per case.
#[test]
fn timing() {
    if std::env::var("ORACLE_TIMING").is_err() {
        return;
    }
    let records = records();
    let defs: Vec<_> = records.iter().map(definition).collect();
    let start = std::time::Instant::now();
    let built: Vec<_> = defs
        .iter()
        .filter_map(|d| construct(d, &Config::default()).ok())
        .collect();
    let construct_time = start.elapsed();
    let start = std::time::Instant::now();
    for (c, r) in built.iter().zip(&records) {
        render(
            c,
            &View::new(0, 0),
            &frame(r),
            &FixedAdvanceMetrics::default(),
        )
        .ok();
    }
    let render_time = start.elapsed();
    let n = built.len() as u32;
    eprintln!(
        "{n} graphics: construct {:?} each, render {:?} each",
        construct_time / n,
        render_time / n
    );
}

/// `CONTACT_SHEET_DIR=<dir> cargo test --test oracle_catalog contact_sheet`
/// writes pages of every base case drawn by milgraphics over the
/// oracle's lines (faint red), for review by eye.
#[test]
fn contact_sheet() {
    let Ok(dir) = std::env::var("CONTACT_SHEET_DIR") else {
        return;
    };
    let cells: Vec<String> = records()
        .iter()
        .filter(|r| {
            let case = r["case"].as_str().unwrap_or_default();
            case.ends_with("-d") || case.ends_with("-e")
        })
        .filter_map(cell)
        .collect();
    for (page, chunk) in cells.chunks(48).enumerate() {
        let mut svg = String::from(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1800"><rect width="1600" height="1800" fill="white"/>"#,
        );
        for (i, c) in chunk.iter().enumerate() {
            let (x, y) = ((i % 8) as f64 * 200.0, (i / 8) as f64 * 300.0);
            svg.push_str(&format!(r#"<g transform="translate({x} {y})">{c}</g>"#));
        }
        svg.push_str("</svg>");
        std::fs::write(format!("{dir}/sheet-{page:02}.svg"), svg).unwrap();
    }
}

/// `CASE_DIR=<dir> CASE_SUFFIX=-app6e cargo test --test oracle_catalog case_cells`
/// writes each base case with that suffix as its own 200×300 cell, for
/// comparing graphics one by one with the standard's plates.
#[test]
fn case_cells() {
    let (Ok(dir), Ok(suffix)) = (std::env::var("CASE_DIR"), std::env::var("CASE_SUFFIX")) else {
        return;
    };
    for r in records() {
        let case = r["case"].as_str().unwrap_or_default();
        if !case.ends_with(&suffix) {
            continue;
        }
        if let Some(c) = cell(&r) {
            let svg = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="300"><rect width="200" height="300" fill="white"/>{c}</svg>"#
            );
            std::fs::write(format!("{dir}/{case}.svg"), svg).unwrap();
        }
    }
}

/// One 200×300 cell: the case drawn in a 200×260 box with its name.
fn cell(r: &Value) -> Option<String> {
    let case = r["case"].as_str()?;
    let symbol = SymbolId::parse(r["symbol"].as_str()?).ok()?;
    let name = support::spec(&symbol).map_or("undeclared", |s| s.name());
    let f = frame(r);
    let oracle = oracle_lines(r, &f);
    let c = construct(&definition(r), &Config::default()).ok()?;
    let plan = render(&c, &View::new(0, 0), &f, &FixedAdvanceMetrics::default()).ok()?;
    let pts = plan.screen.iter().flat_map(|i| i.shape.points().to_vec());
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in pts.chain(oracle.iter().flatten().copied()) {
        (x0, y0, x1, y1) = (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y));
    }
    let (w, h) = ((x1 - x0).max(1.0) + 60.0, (y1 - y0).max(1.0) + 60.0);
    let inner = milgraphics::svg::to_svg(&plan, &milgraphics::svg::SvgOptions::new(4000.0, 4000.0));
    let inner = inner.split_once('\n')?.1.trim_end_matches("</svg>\n");
    let under: String = oracle
        .iter()
        .map(|l| {
            let pts: Vec<String> = l.iter().map(|p| format!("{:.1},{:.1}", p.x, p.y)).collect();
            format!(
                r##"<polyline points="{}" fill="none" stroke="#ff000060" stroke-width="7"/>"##,
                pts.join(" ")
            )
        })
        .collect();
    Some(format!(
        r#"<svg width="200" height="260" viewBox="{} {} {w} {h}" preserveAspectRatio="xMidYMid meet">{under}{inner}</svg><text x="100" y="275" font-size="10" text-anchor="middle" font-family="sans-serif">{case}</text><text x="100" y="290" font-size="9" text-anchor="middle" font-family="sans-serif">{}</text>"#,
        x0 - 30.0,
        y0 - 30.0,
        name.replace('&', "&amp;")
            .chars()
            .take(40)
            .collect::<String>()
    ))
}

/// `ORACLE_CASE=<id> cargo test --test oracle_catalog dump -- --nocapture`
/// prints both sides of one case.
#[test]
fn dump() {
    let Ok(case) = std::env::var("ORACLE_CASE") else {
        return;
    };
    let r = records()
        .into_iter()
        .find(|r| r["case"] == case.as_str())
        .unwrap();
    let f = frame(&r);
    for l in oracle_lines(&r, &f) {
        let pts: Vec<(i64, i64)> = l
            .iter()
            .map(|p| ((p.x * 10.0) as i64, (p.y * 10.0) as i64))
            .collect();
        eprintln!("oracle {pts:?}");
    }
    let c = construct(&definition(&r), &Config::default()).unwrap();
    let plan = render(&c, &View::new(0, 0), &f, &FixedAdvanceMetrics::default()).unwrap();
    for i in &plan.screen {
        let pts: Vec<(i64, i64)> = i
            .shape
            .points()
            .iter()
            .map(|p| ((p.x * 10.0) as i64, (p.y * 10.0) as i64))
            .collect();
        eprintln!("ours{} {pts:?}", if i.decoration { "*" } else { "" });
    }
}
