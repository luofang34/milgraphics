//! Semantic references from drawn output back to the definition.

use crate::construction::PartId;
use crate::definition::GraphicId;
use crate::edit::HandleId;

/// What a drawn item is: which graphic, which part, which handle.
///
/// Map adapters assign their own numeric feature IDs and keep a table from
/// those IDs to `PickRef`, bound to the plan that produced them, so a pick
/// against an outdated plan can be recognised and rejected.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PickRef {
    /// The graphic.
    pub definition: GraphicId,
    /// The part within the graphic.
    pub part: PartId,
    /// The handle, for handle items.
    pub handle: Option<HandleId>,
}
