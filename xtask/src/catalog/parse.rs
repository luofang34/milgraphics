//! Reads the tab-indented symbol tables the way upstream's `MSLookup` does.
//!
//! Columns: 0 symbol set, 1-3 entity / type / subtype names, 4 six-digit
//! entity code, 5 version codes, 6 geometry, 7 draw rule, 8 modifiers (colour
//! hex in the METOC sets), 9 free-form remarks. Names, symbol set and version
//! carry forward from earlier lines exactly as in `MSLookup.populateLookup`.

use super::{Geometry, Row};
use crate::error::XtaskError;

/// Symbol sets whose rows carry geometry and a draw rule.
const MULTIPOINT_SETS: [u8; 3] = [25, 45, 46];
/// Version codes upstream defines for the 2525D/E and APP-6D/E families.
const VERSION_CODES: std::ops::RangeInclusive<u8> = 10..=16;

/// Carry-forward state between lines.
#[derive(Default)]
struct Context {
    symbol_set: String,
    versions: String,
    entity: String,
    entity_type: String,
    entity_subtype: String,
}

/// Rows with geometry and draw rule from one data file, in file order.
///
/// Category rows (no geometry column) are skipped: they have nothing to draw.
pub(super) fn parse_table(file: &'static str, text: &str) -> Result<Vec<Row>, XtaskError> {
    let mut context = Context::default();
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let fail = |message: String| XtaskError::Table {
            file,
            line: index.wrapping_add(1),
            message,
        };
        if let Some(row) = context.step(line).map_err(fail)? {
            rows.push(row);
        }
    }
    Ok(rows)
}

impl Context {
    fn step(&mut self, line: &str) -> Result<Option<Row>, String> {
        // Java's String.split drops trailing empty fields, and MSLookup's
        // length tests are written against that.
        let mut fields: Vec<&str> = line.split('\t').collect();
        while fields.last() == Some(&"") {
            fields.pop();
        }
        let len = fields.len();
        if len < 6 {
            return Err(format!("{len} fields, need at least 6"));
        }
        let col = |i: usize| fields.get(i).copied().unwrap_or("");

        self.entity_subtype = col(3).to_owned();
        if self.entity_subtype.is_empty() {
            self.entity_type = col(2).to_owned();
        }
        if self.entity_type.is_empty() {
            self.entity = col(1).to_owned();
        }
        if !col(0).is_empty() {
            self.symbol_set = col(0).to_owned();
        }
        if !col(5).is_empty() {
            self.versions = col(5).to_owned();
        }

        let symbol_set: u8 = self
            .symbol_set
            .parse()
            .map_err(|_| format!("symbol set {:?} is not a number", self.symbol_set))?;
        let code = if col(4) == "0" { "000000" } else { col(4) };
        if code == "000000" || !MULTIPOINT_SETS.contains(&symbol_set) || len < 8 {
            return Ok(None);
        }
        self.row(symbol_set, code, &fields)
    }

    fn row(&self, symbol_set: u8, code: &str, fields: &[&str]) -> Result<Option<Row>, String> {
        let col = |i: usize| fields.get(i).copied().unwrap_or("");
        let entity: u32 = code
            .parse()
            .map_err(|_| format!("entity code {code:?} is not a number"))?;
        if col(6).is_empty() && col(7).is_empty() {
            // Remarks on a category row can push it past the length threshold.
            return Ok(None);
        }
        let geometry = match col(6) {
            "Point" => Geometry::Point,
            "Line" => Geometry::Line,
            "Area" => Geometry::Area,
            other => return Err(format!("unknown geometry {other:?}")),
        };
        if col(7).is_empty() {
            return Err("geometry without a draw rule".to_owned());
        }
        let name = [&self.entity_subtype, &self.entity_type, &self.entity]
            .into_iter()
            .find(|n| !n.is_empty())
            .cloned()
            .unwrap_or_default();
        let path = if !self.entity_subtype.is_empty() {
            vec![self.entity.clone(), self.entity_type.clone()]
        } else if !self.entity_type.is_empty() {
            vec![self.entity.clone()]
        } else {
            Vec::new()
        };
        Ok(Some(Row {
            symbol_set,
            entity,
            name,
            path: path.into_iter().filter(|n| !n.is_empty()).collect(),
            versions: parse_versions(&self.versions)?,
            geometry,
            draw_rule: col(7).to_owned(),
            modifiers: parse_modifiers(symbol_set, col(8))?,
        }))
    }
}

fn parse_versions(text: &str) -> Result<u32, String> {
    let mut bits = 0u32;
    for part in text.split(',') {
        let code: u8 = part
            .parse()
            .map_err(|_| format!("version {part:?} is not a number"))?;
        if !VERSION_CODES.contains(&code) {
            return Err(format!("version code {code} outside 10..=16"));
        }
        bits |= 1u32 << code;
    }
    Ok(bits)
}

/// Control-measure rows list modifier keys; METOC rows put a colour there,
/// which upstream turns into empty modifier names, so it yields none.
fn parse_modifiers(symbol_set: u8, text: &str) -> Result<Vec<String>, String> {
    if symbol_set != 25 {
        let is_colour = text.chars().all(|c| c.is_ascii_hexdigit() || c == ';');
        return if is_colour {
            Ok(Vec::new())
        } else {
            Err(format!("METOC modifier column {text:?} is not a colour"))
        };
    }
    let mut tokens = Vec::new();
    for token in text.split(',').filter(|t| !t.is_empty()) {
        let letters = token.chars().take_while(char::is_ascii_uppercase).count();
        let digits_ok = token.chars().skip(letters).all(|c| c.is_ascii_digit());
        if letters == 0 || !digits_ok {
            return Err(format!("modifier token {token:?} is not LETTERS[digits]"));
        }
        tokens.push(token.to_owned());
    }
    Ok(tokens)
}
