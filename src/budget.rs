//! Limits on input size and generated output.

use crate::modifier::ModifierField;

/// Limits that bound the work and memory a single graphic may cause.
///
/// Every limit is checked before the work it guards, and exceeding one is a
/// typed error rather than a truncated result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Budget {
    /// Control points per graphic. Control points are indexed by `u32`
    /// (in [`crate::HandleId::Vertex`] and vertex edits), so a larger value
    /// acts as `u32::MAX`.
    pub max_control_points: usize,
    /// Characters in one text amplifier.
    pub max_text_chars: usize,
    /// Values in one multi-valued amplifier (`AM`, `AN`, `X`).
    pub max_modifier_values: usize,
    /// Vertices generated for one graphic, counted separately for its
    /// construction and for each render (geographic and screen tiers
    /// together, decorations and antimeridian cuts included).
    pub max_vertices: usize,
    /// Labels generated for one graphic.
    pub max_labels: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            max_control_points: 10_000,
            max_text_chars: 256,
            max_modifier_values: 64,
            max_vertices: 200_000,
            max_labels: 256,
        }
    }
}

impl Budget {
    /// The control-point limit in force: `max_control_points`, at most
    /// `u32::MAX`.
    pub(crate) fn control_point_limit(&self) -> usize {
        self.max_control_points
            .min(usize::try_from(u32::MAX).unwrap_or(usize::MAX))
    }
}

/// A [`Budget`] limit that an input or its output would exceed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum BudgetError {
    /// Too many control points.
    #[error("{count} control points; the limit is {limit}")]
    ControlPoints {
        /// Points supplied.
        count: usize,
        /// The limit.
        limit: usize,
    },
    /// A text amplifier is too long.
    #[error("amplifier {field} has {count} characters; the limit is {limit}")]
    Text {
        /// The field.
        field: ModifierField,
        /// Characters supplied.
        count: usize,
        /// The limit.
        limit: usize,
    },
    /// A multi-valued amplifier has too many values.
    #[error("amplifier {field} has {count} values; the limit is {limit}")]
    ModifierValues {
        /// The field.
        field: ModifierField,
        /// Values supplied.
        count: usize,
        /// The limit.
        limit: usize,
    },
    /// Generation would exceed the vertex limit.
    #[error("output would need {count} vertices; the limit is {limit}")]
    Vertices {
        /// Vertices needed.
        count: usize,
        /// The limit.
        limit: usize,
    },
    /// Generation would exceed the label limit.
    #[error("output would need {count} labels; the limit is {limit}")]
    Labels {
        /// Labels needed.
        count: usize,
        /// The limit.
        limit: usize,
    },
}

/// Counts vertices as they are generated and stops at the budget.
#[derive(Debug)]
pub(crate) struct VertexMeter {
    used: usize,
    limit: usize,
}

impl VertexMeter {
    pub(crate) fn new(budget: &Budget) -> Self {
        Self {
            used: 0,
            limit: budget.max_vertices,
        }
    }

    /// Reserves `count` more vertices.
    pub(crate) fn take(&mut self, count: usize) -> Result<(), BudgetError> {
        let total = self.used.saturating_add(count);
        if total > self.limit {
            return Err(BudgetError::Vertices {
                count: total,
                limit: self.limit,
            });
        }
        self.used = total;
        Ok(())
    }
}
