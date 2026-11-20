//! Deliberately finite JSON Schema interpreter, written for this experiment.
//! Unknown validation keywords and references are rejected before prompting.
use schemars::{JsonSchema, schema_for};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn contract<T: JsonSchema>() -> Result<Value, String> {
    let schema = serde_json::to_value(schema_for!(T)).map_err(|e| e.to_string())?;
    supported(&schema)?;
    Ok(schema)
}

pub fn supported(s: &Value) -> Result<(), String> {
    let obj = s.as_object().ok_or("schema must be an object")?;
    for (key, value) in obj {
        match key.as_str() {
            "$schema" | "title" | "description" => {}
            "type"
                if matches!(
                    value.as_str(),
                    Some("object" | "array" | "string" | "integer" | "number" | "boolean" | "null")
                ) => {}
            "required"
                if value
                    .as_array()
                    .is_some_and(|a| a.iter().all(Value::is_string)) => {}
            "const" => {}
            "enum" if value.is_array() => {}
            "additionalProperties" if value.is_boolean() => {}
            "properties" => {
                for sub in value.as_object().ok_or("invalid properties")?.values() {
                    supported(sub)?;
                }
            }
            "items" => supported(value)?,
            "oneOf" | "anyOf" => {
                for sub in value.as_array().ok_or("invalid alternatives")? {
                    supported(sub)?;
                }
            }
            "format" if value == "int64" => {}
            _ => return Err(format!("unsupported schema keyword/value: {key}")),
        }
    }
    Ok(())
}

pub fn validate(s: &Value, v: &Value) -> Result<(), String> {
    if let Some(kind) = s["type"].as_str() {
        let matches = match kind {
            "object" => v.is_object(),
            "array" => v.is_array(),
            "string" => v.is_string(),
            "integer" => v.is_i64() || v.is_u64(),
            "number" => v.is_number(),
            "boolean" => v.is_boolean(),
            "null" => v.is_null(),
            _ => false,
        };
        if !matches {
            return Err(format!("expected {kind}"));
        }
    }
    if s["format"] == "int64" && !v.is_i64() {
        return Err("expected signed 64-bit integer".into());
    }
    if s.get("const").is_some_and(|c| c != v) {
        return Err("incorrect constant".into());
    }
    if s["enum"].as_array().is_some_and(|a| !a.contains(v)) {
        return Err("not an enum member".into());
    }
    for key in ["oneOf", "anyOf"] {
        if let Some(options) = s[key].as_array() {
            let matches = options.iter().filter(|s| validate(s, v).is_ok()).count();
            if matches == 0 || (key == "oneOf" && matches != 1) {
                return Err(format!("violates {key}"));
            }
        }
    }
    if let Some(obj) = v.as_object() {
        if let Some(required) = s["required"].as_array() {
            for field in required {
                if !obj.contains_key(field.as_str().ok_or("invalid required")?) {
                    return Err(format!("missing {field}"));
                }
            }
        }
        for (key, value) in obj {
            if let Some(sub) = s["properties"].get(key) {
                validate(sub, value).map_err(|e| format!("{key}: {e}"))?;
            } else if s["additionalProperties"] == false {
                return Err(format!("unknown field {key}"));
            }
        }
    }
    if let (Some(items), Some(values)) = (s.get("items"), v.as_array()) {
        for value in values {
            validate(items, value)?;
        }
    }
    Ok(())
}

/// Validation and conversion both run before acknowledging a candidate.
pub fn decode<T: DeserializeOwned>(s: &Value, v: Value) -> Result<T, String> {
    validate(s, &v)?;
    serde_json::from_value(v).map_err(|e| e.to_string())
}
