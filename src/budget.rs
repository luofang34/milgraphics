//! Limits on input size and generated output.

/// Limits that bound the work and memory a single graphic may cause.
///
/// Every limit is checked before the work it guards, and exceeding one is a
/// typed error rather than a truncated result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    /// Control points per graphic.
    pub max_control_points: usize,
    /// Characters in one text amplifier.
    pub max_text_chars: usize,
    /// Values in one multi-valued amplifier (`AM`, `AN`, `X`).
    pub max_modifier_values: usize,
    /// Vertices generated for one graphic, over all parts and both tiers.
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

/// A [`Budget`] limit that an input or its output would exceed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
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
        /// Field letter, e.g. `T`.
        field: &'static str,
        /// Characters supplied.
        count: usize,
        /// The limit.
        limit: usize,
    },
    /// A multi-valued amplifier has too many values.
    #[error("amplifier {field} has {count} values; the limit is {limit}")]
    ModifierValues {
        /// Field letter, e.g. `AM`.
        field: &'static str,
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
