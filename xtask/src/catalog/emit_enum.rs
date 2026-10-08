//! Emits a documented, name-addressable enum as Rust source.

use std::fmt::Write;

use crate::error::XtaskError;

/// Arms and variants written per output line, to keep generated functions short.
const PER_LINE: usize = 5;

/// One enum variant.
pub(super) struct Variant {
    /// Rust identifier.
    pub(super) ident: String,
    /// Upstream spelling returned by `name()`.
    pub(super) name: String,
    /// Doc comment text.
    pub(super) doc: String,
}

/// Appends `pub enum <name>` with `ALL`, `name()` and `from_name()`.
pub(super) fn emit_enum(
    out: &mut String,
    name: &str,
    doc: &str,
    variants: &[Variant],
) -> Result<(), XtaskError> {
    writeln!(out, "/// {doc}")?;
    writeln!(
        out,
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]"
    )?;
    writeln!(out, "#[non_exhaustive]")?;
    writeln!(out, "pub enum {name} {{")?;
    for v in variants {
        writeln!(out, "    /// {}", v.doc)?;
        writeln!(out, "    {},", v.ident)?;
    }
    writeln!(out, "}}\n\nimpl {name} {{")?;
    writeln!(out, "    /// Every variant, in declaration order.")?;
    writeln!(out, "    pub const ALL: &[Self] = &[")?;
    for chunk in variants.chunks(PER_LINE) {
        let line: Vec<String> = chunk.iter().map(|v| format!("Self::{}", v.ident)).collect();
        writeln!(out, "        {},", line.join(", "))?;
    }
    writeln!(out, "    ];\n")?;
    writeln!(out, "    /// The upstream spelling of this variant.")?;
    writeln!(out, "    pub const fn name(self) -> &'static str {{")?;
    writeln!(out, "        match self {{")?;
    for chunk in variants.chunks(PER_LINE) {
        let line: Vec<String> = chunk
            .iter()
            .map(|v| format!("Self::{} => {:?}", v.ident, v.name))
            .collect();
        writeln!(out, "            {},", line.join(", "))?;
    }
    writeln!(out, "        }}\n    }}\n")?;
    writeln!(
        out,
        "    /// The variant whose upstream spelling is `name`, if any."
    )?;
    writeln!(out, "    pub fn from_name(name: &str) -> Option<Self> {{")?;
    writeln!(
        out,
        "        Self::ALL.iter().copied().find(|item| item.name() == name)"
    )?;
    writeln!(out, "    }}\n}}")?;
    Ok(())
}
