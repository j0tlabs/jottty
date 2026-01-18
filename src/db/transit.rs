use serde_json::Value;
// TODO@chico: switch to a more efficient format like
// Binary JSON (BSON) or MessagePack or CBOR.

/// Encode a serde_json::Value into a JSON string.
/// Returns a Result containing the JSON string or an error.
pub fn encode_value(value: &Value) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Decode a JSON string into a serde_json::Value.
/// Returns a Result containing the Value or an error.
pub fn decode_value(input: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(input)
}
