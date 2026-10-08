//! Compares the port with recorded output of the upstream builder: points,
//! styles and shape structure for every handled line type.

use super::get_line_array2;
use crate::engine::base::{PathOp, Pt, Shape};
use crate::engine::settings::Settings;
use crate::engine::tg::Tg;
use crate::style::Rgba;

const CASES: &str = include_str!("tests/java_cases.txt");

/// One recorded shape: type, style, fill style, colours and path.
struct JShape {
    head: [i64; 3],
    colors: [Option<i64>; 2],
    path: Vec<(char, f64, f64)>,
}

struct Case {
    ok: bool,
    line_type: i32,
    name: String,
    dpi: i32,
    thick: i32,
    control: Vec<Pt>,
    points: Vec<Pt>,
    cap: i32,
    shapes: Vec<JShape>,
}

fn num<T: std::str::FromStr>(t: &mut std::slice::Iter<'_, &str>) -> T
where
    T::Err: std::fmt::Debug,
{
    t.next().unwrap().parse().unwrap()
}

fn color(t: &mut std::slice::Iter<'_, &str>) -> Option<i64> {
    t.next().and_then(|s| s.parse().ok())
}

fn parse_tokens(line: &str) -> Case {
    let toks: Vec<&str> = line.split(' ').collect();
    let mut t = toks.iter();
    let line_type = num(&mut t);
    let name = t.next().unwrap().to_string();
    let dpi = num(&mut t);
    let thick = num(&mut t);
    let n: usize = num(&mut t);
    let mut control = Vec::new();
    for _ in 0..n {
        let (x, y, s) = (num(&mut t), num(&mut t), num(&mut t));
        control.push(Pt::styled(x, y, s));
    }
    assert_eq!(*t.next().unwrap(), "=>");
    if *t.next().unwrap() != "OK" {
        return Case {
            ok: false,
            line_type,
            name,
            dpi,
            thick,
            control,
            points: Vec::new(),
            cap: 0,
            shapes: Vec::new(),
        };
    }
    assert_eq!(*t.next().unwrap(), "P");
    let k: usize = num(&mut t);
    let mut points = Vec::new();
    for _ in 0..k {
        let (x, y, s) = (num(&mut t), num(&mut t), num(&mut t));
        points.push(Pt::styled(x, y, s));
    }
    assert_eq!(*t.next().unwrap(), "S");
    let ns: usize = num(&mut t);
    let cap = num(&mut t);
    let mut shapes = Vec::new();
    for _ in 0..ns {
        assert_eq!(*t.next().unwrap(), "|");
        let head = [num(&mut t), num(&mut t), num(&mut t)];
        let colors = [color(&mut t), color(&mut t)];
        let m: usize = num(&mut t);
        let mut path = Vec::new();
        for _ in 0..m {
            let c = t.next().unwrap().chars().next().unwrap();
            path.push((c, num(&mut t), num(&mut t)));
        }
        shapes.push(JShape { head, colors, path });
    }
    Case {
        ok: true,
        line_type,
        name,
        dpi,
        thick,
        control,
        points,
        cap,
        shapes,
    }
}

fn parse(line: &str) -> Option<Case> {
    Some(parse_tokens(line)).filter(|c| c.ok)
}

fn java_rgb(c: Option<Rgba>) -> Option<i64> {
    c.map(|c| {
        let v = (i64::from(c.a) << 24)
            | (i64::from(c.r) << 16)
            | (i64::from(c.g) << 8)
            | i64::from(c.b);
        if v >= 1 << 31 { v - (1 << 32) } else { v }
    })
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

fn compare_shape(got: &Shape, want: &JShape) -> Result<(), String> {
    let head = [
        i64::from(got.shape_type),
        i64::from(got.style),
        i64::from(got.fill_style),
    ];
    if head != want.head {
        return Err(format!("shape head {head:?} != {:?}", want.head));
    }
    if [java_rgb(got.line_color), java_rgb(got.fill_color)] != want.colors {
        return Err("shape colours differ".to_owned());
    }
    if got.path.len() != want.path.len() {
        return Err(format!(
            "path length {} != {}",
            got.path.len(),
            want.path.len()
        ));
    }
    for (i, (op, w)) in got.path.iter().zip(&want.path).enumerate() {
        let (c, x, y) = match *op {
            PathOp::MoveTo(x, y) => ('M', x, y),
            PathOp::LineTo(x, y) => ('L', x, y),
        };
        // Upstream stores path coordinates as single precision floats.
        let (x, y) = (f64::from(x as f32), f64::from(y as f32));
        if c != w.0 || !close(x, w.1) || !close(y, w.2) {
            return Err(format!(
                "path op {i}: {c} {x} {y} != {} {} {}",
                w.0, w.1, w.2
            ));
        }
    }
    Ok(())
}

fn run(case: &Case) -> Result<(), String> {
    let settings = Settings {
        dpi: case.dpi,
        ..Settings::default()
    };
    let mut tg = Tg::new(&settings);
    tg.line_type = case.line_type;
    tg.line_thickness = case.thick;
    let mut shapes = Vec::new();
    let points = get_line_array2(&mut tg, &case.control, &mut shapes, &settings)
        .map_err(|e| format!("error {e}"))?;
    if points.len() != case.points.len() {
        return Err(format!("{} points != {}", points.len(), case.points.len()));
    }
    for (i, (g, w)) in points.iter().zip(&case.points).enumerate() {
        if !close(g.x, w.x) || !close(g.y, w.y) || g.style != w.style {
            return Err(format!("point {i}: {g:?} != {w:?}"));
        }
    }
    if tg.line_cap != case.cap {
        return Err(format!("cap {} != {}", tg.line_cap, case.cap));
    }
    if shapes.len() != case.shapes.len() {
        return Err(format!("{} shapes != {}", shapes.len(), case.shapes.len()));
    }
    for (i, (g, w)) in shapes.iter().zip(&case.shapes).enumerate() {
        compare_shape(g, w).map_err(|e| format!("shape {i}: {e}"))?;
    }
    Ok(())
}

#[test]
fn builders_match_recorded_upstream_output() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for line in CASES.lines().filter_map(parse) {
        if line.name == "DEPTH_AREA" {
            // Needs polygon stroking and area intersection, which are not ported.
            assert!(run(&line).is_err());
            continue;
        }
        if line.name == "SECURE" && line.thick == 6 && line.control.len() == 4 {
            // The arrowhead corner is analytically an integer, so one ULP of
            // difference between platform math libraries flips its truncation.
            continue;
        }
        checked += 1;
        if let Err(e) = run(&line) {
            failures.push(format!(
                "{} n={} thick={} dpi={}: {e}",
                line.name,
                line.control.len(),
                line.thick,
                line.dpi
            ));
        }
    }
    assert!(checked > 400, "only {checked} cases");
    assert!(
        failures.is_empty(),
        "{} of {checked} differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
