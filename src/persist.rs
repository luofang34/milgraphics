//! The persistence envelope: stored JSON kept intact, decoded on demand.

use serde_json::value::RawValue;
use serde_json::{Map, Value};

use crate::definition::GraphicDefinition;

#[cfg(test)]
mod tests;

/// Schema version this library writes and decodes.
pub const SCHEMA_VERSION: u64 = 1;

/// A stored graphic exactly as it was read.
///
/// The original bytes are kept, so a graphic this version cannot decode —
/// newer schema, unknown standard, unsupported symbol — is written back
/// byte for byte. Decoding produces a [`GraphicDefinition`] whose unknown
/// fields keep their JSON content through edits.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct PersistedGraphic {
    raw: Box<RawValue>,
}

/// Why stored JSON could not be accepted or decoded.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PersistError {
    /// The input is not a JSON object.
    #[error("stored graphic is not a JSON object: {0}")]
    NotAnObject(#[source] serde_json::Error),
    /// The input is a JSON value other than an object.
    #[error("stored graphic is a JSON {found}, not an object")]
    WrongType {
        /// The JSON type found.
        found: &'static str,
    },
    /// The `schema` field is missing or not one this version decodes.
    #[error("stored graphic has schema {found:?}; this version decodes schema {SCHEMA_VERSION}")]
    UnknownSchema {
        /// The `schema` value found, if any.
        found: Option<Value>,
    },
    /// The object does not match the definition format.
    #[error("stored graphic does not match schema {SCHEMA_VERSION}: {0}")]
    Invalid(#[source] serde_json::Error),
    /// A definition could not be serialized.
    #[error("graphic {id} could not be serialized: {source}")]
    Serialize {
        /// ID of the definition.
        id: String,
        /// The serializer error.
        #[source]
        source: serde_json::Error,
    },
}

impl PersistedGraphic {
    /// Accepts any JSON object, whether or not this version can decode it.
    pub fn from_json(json: &str) -> Result<Self, PersistError> {
        let raw: Box<RawValue> = serde_json::from_str(json).map_err(PersistError::NotAnObject)?;
        let found = json_type(raw.get());
        if found != "object" {
            return Err(PersistError::WrongType { found });
        }
        Ok(Self { raw })
    }

    /// The stored JSON, exactly as read or written.
    pub fn as_json(&self) -> &str {
        self.raw.get()
    }

    /// Encodes a definition with the current schema version.
    pub fn from_definition(definition: &GraphicDefinition) -> Result<Self, PersistError> {
        let serialize = |source| PersistError::Serialize {
            id: definition.id.to_string(),
            source,
        };
        let mut object = match serde_json::to_value(definition).map_err(serialize)? {
            Value::Object(object) => object,
            _ => Map::new(),
        };
        object.insert("schema".to_owned(), Value::from(SCHEMA_VERSION));
        let json = serde_json::to_string(&object).map_err(serialize)?;
        let raw = RawValue::from_string(json).map_err(serialize)?;
        Ok(Self { raw })
    }

    /// Decodes the definition, or explains why this version cannot.
    ///
    /// A decode failure does not affect the stored JSON: the host keeps this
    /// value and writes it back unchanged.
    pub fn decode(&self) -> Result<GraphicDefinition, PersistError> {
        let mut object: Map<String, Value> =
            serde_json::from_str(self.raw.get()).map_err(PersistError::Invalid)?;
        match object.remove("schema") {
            Some(v) if v.as_u64() == Some(SCHEMA_VERSION) => {}
            found => return Err(PersistError::UnknownSchema { found }),
        }
        serde_json::from_value(Value::Object(object)).map_err(PersistError::Invalid)
    }
}

/// The JSON type of a syntactically valid JSON text.
fn json_type(json: &str) -> &'static str {
    match json.trim_start().bytes().next() {
        Some(b'{') => "object",
        Some(b'[') => "array",
        Some(b'"') => "string",
        Some(b't' | b'f') => "boolean",
        Some(b'n') => "null",
        _ => "number",
    }
}
