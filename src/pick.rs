//! Semantic references from drawn output back to the definition.

use crate::construction::PartId;
use crate::definition::GraphicId;
use crate::edit::HandleId;

/// What a drawn item is: which graphic, and which part or handle of it.
///
/// Map adapters assign their own numeric feature IDs and keep a table from
/// those IDs to `PickRef`, bound to the plan that produced them, so a pick
/// against an outdated plan can be recognised and rejected.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct PickRef {
    /// The graphic.
    pub definition: GraphicId,
    /// The part or handle within it.
    pub target: PickTarget,
}

/// The element of a graphic a pick refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PickTarget {
    /// A drawn part (line, boundary, arrowhead, label of that part, …).
    Part(PartId),
    /// An edit handle.
    Handle(HandleId),
}
