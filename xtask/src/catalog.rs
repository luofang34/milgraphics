//! `cargo xtask catalog`: extracts the symbol catalog and draw-rule names from
//! the pinned mil-sym-java files into `src/generated/`.

mod emit;
mod emit_enum;
mod java_consts;
mod parse;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::XtaskError;
use emit_enum::Variant;
use java_consts::{JavaConst, parse_consts};

/// Directory of the pinned clone, relative to the workspace root.
const UPSTREAM: &str = "tools/oracle/upstream/mil-sym-java";
const DATA_D: &str = "src/main/resources/data/msd.txt";
const DATA_E: &str = "src/main/resources/data/mse.txt";
const RULES: &str = "src/main/java/armyc2/c5isr/renderer/utilities/DrawRules.java";
const MO_RULES: &str = "src/main/java/armyc2/c5isr/renderer/utilities/MODrawRules.java";

/// Geometry column of a data row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Geometry {
    Point,
    Line,
    Area,
}

/// One data row with geometry and a draw rule.
#[derive(Debug)]
pub(crate) struct Row {
    pub(crate) symbol_set: u8,
    pub(crate) entity: u32,
    name: String,
    path: Vec<String>,
    /// Bit `n` set when version code `n` is listed.
    pub(crate) versions: u32,
    geometry: Geometry,
    pub(crate) draw_rule: String,
    pub(crate) modifiers: Vec<String>,
}

impl Row {
    fn first_version(&self) -> u32 {
        self.versions.trailing_zeros()
    }
}

/// Regenerates `src/generated/{draw_rule,catalog}.rs`; returns a summary line.
pub(crate) fn run() -> Result<String, XtaskError> {
    let root = root()?;
    let upstream = root.join(UPSTREAM);
    let read = |rel: &str| read_file(&upstream.join(rel));
    let rows = rows(&root)?;

    let standard = parse_consts(&read(RULES)?);
    let metoc = parse_consts(&read(MO_RULES)?);
    check_rule_names(&rows, &standard, &metoc)?;

    let modifiers: BTreeSet<&str> = rows
        .iter()
        .flat_map(|r| r.modifiers.iter().map(String::as_str))
        .collect();
    let draw_rule_src = emit::draw_rule_module(&standard, &metoc)?;
    let catalog_src = emit::catalog_module(&rows, &modifiers)?;
    write_file(&root.join("src/generated/draw_rule.rs"), &draw_rule_src)?;
    write_file(&root.join("src/generated/catalog.rs"), &catalog_src)?;
    Ok(summary(&rows, &standard, &metoc, &modifiers))
}

/// The workspace root.
pub(crate) fn root() -> Result<PathBuf, XtaskError> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| XtaskError::Root(manifest.to_path_buf()))
}

/// The catalog rows, as `src/generated/catalog.rs` lists them.
pub(crate) fn rows(root: &Path) -> Result<Vec<Row>, XtaskError> {
    let upstream = root.join(UPSTREAM);
    let read = |rel: &str| read_file(&upstream.join(rel));
    let mut rows = parse::parse_table("msd.txt", &read(DATA_D)?)?;
    rows.extend(parse::parse_table("mse.txt", &read(DATA_E)?)?);
    select(resolve_overrides(rows))
}

pub(crate) fn read_file(path: &Path) -> Result<String, XtaskError> {
    fs::read_to_string(path).map_err(|source| XtaskError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn write_file(path: &PathBuf, text: &str) -> Result<(), XtaskError> {
    fs::write(path, text).map_err(|source| XtaskError::Write {
        path: path.clone(),
        source,
    })
}

/// Upstream stores each `(symbol set, entity, version)` in a map, so when two
/// rows claim the same triple the later row wins. Mirroring that keeps
/// `lookup(version, set, entity)` independent of row order.
fn resolve_overrides(mut rows: Vec<Row>) -> Vec<Row> {
    let mut owner: BTreeMap<(u8, u32, u32), usize> = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        for version in (0..32).filter(|v| row.versions >> v & 1 == 1) {
            owner.insert((row.symbol_set, row.entity, version), index);
        }
    }
    for (index, row) in rows.iter_mut().enumerate() {
        let kept = (0..32)
            .filter(|v| row.versions >> v & 1 == 1)
            .filter(|v| owner.get(&(row.symbol_set, row.entity, *v)) == Some(&index))
            .fold(0u32, |bits, v| bits | 1u32 << v);
        row.versions = kept;
    }
    rows.retain(|row| row.versions != 0);
    rows
}

/// Keeps symbol set 25 in full and the METOC rows drawn from several points
/// (lines, areas and the two-point `Point5`), ordered by `(symbol set,
/// entity, first version)`.
fn select(mut rows: Vec<Row>) -> Result<Vec<Row>, XtaskError> {
    rows.retain(|r| r.symbol_set == 25 || r.geometry != Geometry::Point || r.draw_rule == "Point5");
    rows.sort_by_key(|r| (r.symbol_set, r.entity, r.first_version()));
    let mut seen = BTreeSet::new();
    for row in &rows {
        for version in (0..32).filter(|v| row.versions >> v & 1 == 1) {
            if !seen.insert((row.symbol_set, row.entity, version)) {
                return Err(XtaskError::Invariant(format!(
                    "duplicate catalog key {}/{}/v{version}",
                    row.symbol_set, row.entity
                )));
            }
        }
    }
    Ok(rows)
}

/// Every draw-rule string in the data must be a constant upstream declares;
/// upstream itself silently maps unknown names to "do not draw".
fn check_rule_names(
    rows: &[Row],
    standard: &[JavaConst],
    metoc: &[JavaConst],
) -> Result<(), XtaskError> {
    for row in rows {
        let known = if row.symbol_set == 25 {
            standard
        } else {
            metoc
        };
        if !known.iter().any(|c| c.name == row.draw_rule) {
            return Err(XtaskError::Invariant(format!(
                "{}/{} uses draw rule {:?}, not declared upstream",
                row.symbol_set, row.entity, row.draw_rule
            )));
        }
    }
    Ok(())
}

fn summary(
    rows: &[Row],
    standard: &[JavaConst],
    metoc: &[JavaConst],
    mods: &BTreeSet<&str>,
) -> String {
    let count = |set: u8| rows.iter().filter(|r| r.symbol_set == set).count();
    format!(
        "catalog: {} entries (set 25: {}, set 45: {}, set 46: {}); {} draw rules, {} METOC draw rules, {} modifier keys",
        rows.len(),
        count(25),
        count(45),
        count(46),
        standard.len(),
        metoc.len(),
        mods.len()
    )
}

/// Enum variants for a list of Java constants.
fn rule_variants(consts: &[JavaConst]) -> Vec<Variant> {
    consts
        .iter()
        .map(|c| Variant {
            ident: c.name.clone(),
            name: c.name.clone(),
            doc: format!("Upstream draw rule `{}`.", c.java_name),
        })
        .collect()
}
