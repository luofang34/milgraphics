//! Checks a definition against its symbol's declaration and the budget
//! before anything is constructed.

use crate::budget::{Budget, BudgetError};
use crate::definition::GraphicDefinition;
use crate::modifier::{ModifierField, ModifierKind, ModifierValue};
use crate::support::SymbolSpec;

use super::ConstructError;

pub(super) fn validate(
    spec: &SymbolSpec,
    def: &GraphicDefinition,
    budget: &Budget,
) -> Result<(), ConstructError> {
    let count = def.points.len();
    let limit = budget.control_point_limit();
    if count > limit {
        return Err(BudgetError::ControlPoints { count, limit }.into());
    }
    if count < spec.min_points || count > spec.max_points {
        return Err(ConstructError::PointCount {
            symbol: spec.name(),
            count,
            min: spec.min_points,
            max: spec.max_points,
        });
    }
    if let Some(index) = def.points.iter().position(|p| p.altitude.is_some()) {
        return Err(ConstructError::UnsupportedAltitude {
            symbol: spec.name(),
            index,
        });
    }
    if let Some(key) = def.modifiers.unknown.keys().next() {
        return Err(ConstructError::UnknownModifier {
            symbol: spec.name(),
            key: key.clone(),
        });
    }
    for &field in ModifierField::ALL {
        check_field(spec, def, budget, field)?;
    }
    Ok(())
}

/// One field: drawn by the symbol if set, present if required, within the
/// declared count and the budget, and with usable values.
fn check_field(
    spec: &SymbolSpec,
    def: &GraphicDefinition,
    budget: &Budget,
    field: ModifierField,
) -> Result<(), ConstructError> {
    let symbol = spec.name();
    let value = def.modifiers.get(field);
    let Some(declared) = spec.modifier(field) else {
        return match value {
            Some(_) => Err(ConstructError::UnsupportedModifier { symbol, field }),
            None => Ok(()),
        };
    };
    let Some(value) = value else {
        return match declared.required {
            true => Err(ConstructError::MissingModifier { symbol, field }),
            false => Ok(()),
        };
    };
    let count = def.modifiers.count(field);
    if count > budget.max_modifier_values {
        let limit = budget.max_modifier_values;
        return Err(BudgetError::ModifierValues {
            field,
            count,
            limit,
        }
        .into());
    }
    if count < declared.min_count || count > declared.max_count {
        return Err(ConstructError::ModifierCount {
            symbol,
            field,
            count,
            min: declared.min_count,
            max: declared.max_count,
        });
    }
    check_values(field, &value, budget)
}

fn check_values(
    field: ModifierField,
    value: &ModifierValue,
    budget: &Budget,
) -> Result<(), ConstructError> {
    let invalid = |index, value, reason| ConstructError::InvalidModifier {
        field,
        index,
        value,
        reason,
    };
    match value {
        ModifierValue::Text(text) => {
            let count = text.chars().count();
            if count > budget.max_text_chars {
                let limit = budget.max_text_chars;
                return Err(BudgetError::Text {
                    field,
                    count,
                    limit,
                }
                .into());
            }
        }
        ModifierValue::Numbers(values) => {
            let non_negative = field.kind() == ModifierKind::Distances;
            for (index, &v) in values.iter().enumerate() {
                if !v.is_finite() {
                    return Err(invalid(index, v, "must be finite"));
                }
                if non_negative && v < 0.0 {
                    return Err(invalid(index, v, "distances must be non-negative"));
                }
            }
        }
        ModifierValue::Altitudes(values) => {
            for (index, a) in values.iter().enumerate() {
                if !a.metres.is_finite() {
                    return Err(invalid(index, a.metres, "must be finite"));
                }
            }
        }
    }
    Ok(())
}
