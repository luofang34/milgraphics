//! Extracts the draw-rule constants declared by upstream's Java classes.

/// One `public static final int NAME = value;` declaration.
#[derive(Debug)]
pub(super) struct JavaConst {
    /// Upper-case Java identifier, e.g. `AREA1`.
    pub(super) java_name: String,
    /// Upstream spelling used in the data files, e.g. `Area1`.
    pub(super) name: String,
}

/// Declarations in file order.
pub(super) fn parse_consts(text: &str) -> Vec<JavaConst> {
    text.lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("public static final int ")?;
            let (ident, _) = rest.split_once('=')?;
            let java_name = ident.trim().to_owned();
            let name = camel_case(&java_name);
            Some(JavaConst { java_name, name })
        })
        .collect()
}

/// `AREA1` becomes `Area1`; the one multi-word constant is spelled out.
pub(super) fn camel_case(java_name: &str) -> String {
    if java_name == "DONOTDRAW" {
        return "DoNotDraw".to_owned();
    }
    let mut chars = java_name.chars();
    match chars.next() {
        Some(first) => {
            let mut out: String = first.to_uppercase().collect();
            out.extend(chars.flat_map(char::to_lowercase));
            out
        }
        None => String::new(),
    }
}
