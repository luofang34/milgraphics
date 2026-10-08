//! Repository automation: `cargo xtask catalog` regenerates the catalog in
//! `src/generated/` from the pinned mil-sym-java data files, and `cargo xtask
//! references` the standard references from `tools/oracle/references.json`.

mod catalog;
mod error;
mod references;

use std::error::Error;
use std::io::Write;

use error::XtaskError;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["catalog"] => {
            let summary = catalog::run()?;
            writeln!(std::io::stdout(), "{summary}")?;
            Ok(())
        }
        ["references"] => {
            let summary = references::run()?;
            writeln!(std::io::stdout(), "{summary}")?;
            Ok(())
        }
        _ => Err(XtaskError::Usage.into()),
    }
}
