/// Version of the construction and layout rules that produced an output.
///
/// Recorded in every construction and render plan so a result can be
/// reproduced and diagnosed, and part of every cache key: output from a
/// different renderer version is never reused.
pub const RENDERER_VERSION: &str = env!("CARGO_PKG_VERSION");
