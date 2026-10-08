//! Errors reported by the repository automation.

use std::path::PathBuf;

use thiserror::Error;

/// Everything `cargo xtask` can fail with.
#[derive(Debug, Error)]
pub(crate) enum XtaskError {
    /// Unknown or missing subcommand.
    #[error("usage: cargo xtask catalog")]
    Usage,
    /// The workspace root could not be derived from the manifest directory.
    #[error("cannot locate the workspace root from {0}")]
    Root(PathBuf),
    /// An input file could not be read.
    #[error("cannot read {path} (run tools/oracle/fetch-upstream.sh first): {source}")]
    Read {
        /// File that failed to open.
        path: PathBuf,
        /// Underlying I/O failure.
        #[source]
        source: std::io::Error,
    },
    /// An output file could not be written.
    #[error("cannot write {path}: {source}")]
    Write {
        /// File that failed to write.
        path: PathBuf,
        /// Underlying I/O failure.
        #[source]
        source: std::io::Error,
    },
    /// A data line does not have the layout upstream's `MSLookup` expects.
    #[error("{file}:{line}: {message}")]
    Table {
        /// Data file name.
        file: &'static str,
        /// One-based line number.
        line: usize,
        /// What is wrong with the line.
        message: String,
    },
    /// The extracted data violates an invariant the generator relies on.
    #[error("{0}")]
    Invariant(String),
    /// Formatting generated source failed.
    #[error("formatting generated source: {0}")]
    Format(#[from] std::fmt::Error),
}
