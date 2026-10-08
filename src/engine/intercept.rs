//! Port of mil-sym-java web/render/WebRenderer.java `interceptAndAdjustCode`:
//! APP-6 codes that duplicate another graphic drawn as a feint or dummy are
//! drawn as that graphic with the feint/dummy indicator.

use crate::sidc::SymbolId;

/// Upstream `SymbolID.HQTFD_FeintDummy`.
const FEINT_DUMMY: char = '1';

/// The duplicate entity codes and the graphic each is drawn as.
const DUPLICATES: [(u32, u32); 6] = [
    // Axis of Advance for a Feint: Supporting Attack.
    (151_406, 151_404),
    // Direction of Attack Feint: Direction of Supporting Attack.
    (140_605, 140_603),
    // Dummy Minefield: Static Depiction.
    (270_705, 270_701),
    // Dummy Minefield, Dynamic: Dynamic Depiction.
    (270_706, 270_707),
    // Decoy Mined Area: Mined Area.
    (270_900, 270_800),
    // Decoy Mined Area, Fenced: Mined Area, Fenced.
    (270_901, 270_801),
];

/// The code the renderer draws `symbol` as: itself, or for a duplicate the
/// graphic it duplicates, marked as a feint or dummy.
pub(crate) fn engine_symbol(symbol: &SymbolId) -> SymbolId {
    let entity = symbol.entity().get();
    let target = DUPLICATES
        .iter()
        .find(|(from, _)| symbol.symbol_set() == 25 && *from == entity)
        .map(|&(_, to)| to);
    let Some(target) = target else {
        return symbol.clone();
    };
    let entity_digits = format!("{target:06}");
    let digits: String = symbol
        .as_str()
        .chars()
        .enumerate()
        .map(|(i, c)| match i {
            7 => FEINT_DUMMY,
            10..=15 => entity_digits.chars().nth(i - 10).unwrap_or(c),
            _ => c,
        })
        .collect();
    SymbolId::parse(&digits).unwrap_or_else(|_| symbol.clone())
}

#[cfg(test)]
mod tests;
