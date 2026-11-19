//! Finite contract validation; every accepted keyword is checked independently of Serde.
//! Unknown validation keywords and references are rejected before prompting.
use schemars::{JsonSchema, schema_for};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn contract<T: JsonSchema>() -> crate::Result<Value> {
    let schema = serde_json::to_value(schema_for!(T))
        .map_err(|e| crate::Error::new(crate::ErrorKind::Contract, e.to_string()))?;
    supported(&schema).map_err(|e| crate::Error::new(crate::ErrorKind::Contract, e))?;
    Ok(schema)
}

pub fn supported(s: &Value) -> Result<(), String> {
    let obj = s.as_object().ok_or("schema must be an object")?;
    for (key, value) in obj {
        match key.as_str() {
            "$schema" if value == "https://json-schema.org/draft/2020-12/schema" => {}
            "title" | "description" if value.is_string() => {}
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
            "enum" if value.as_array().is_some_and(|a| !a.is_empty()) => {}
            "additionalProperties" if value.is_boolean() => {}
            "properties" => {
                for (name, sub) in value.as_object().ok_or("invalid properties")? {
                    supported(sub).map_err(|e| format!("/properties/{name}{e}"))?;
                }
            }
            "items" => supported(value).map_err(|e| format!("/items{e}"))?,
            "oneOf" | "anyOf" => {
                if value.as_array().is_none_or(|a| a.is_empty()) {
                    return Err(format!("/{key}: empty alternatives"));
                }

                for (i, sub) in value
                    .as_array()
                    .ok_or("invalid alternatives")?
                    .iter()
                    .enumerate()
                {
                    supported(sub).map_err(|e| format!("/{key}/{i}{e}"))?;
                }
            }
            "format" if value == "int64" && s["type"] == "integer" => {}
            _ => return Err(format!("/{key}: unsupported schema keyword/value")),
        }
    }
    Ok(())
}

fn validate_inner(s: &Value, v: &Value) -> Result<(), String> {
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
    if s.get("const").is_some_and(|c| !equal(c, v)) {
        return Err("incorrect constant".into());
    }
    if s["enum"]
        .as_array()
        .is_some_and(|a| !a.iter().any(|c| equal(c, v)))
    {
        return Err("not an enum member".into());
    }
    for key in ["oneOf", "anyOf"] {
        if let Some(options) = s[key].as_array() {
            let matches = options
                .iter()
                .filter(|s| validate_inner(s, v).is_ok())
                .count();
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
                validate_inner(sub, value).map_err(|e| format!("{key}: {e}"))?;
            } else if s["additionalProperties"] == false {
                return Err(format!("unknown field {key}"));
            }
        }
    }
    if let (Some(items), Some(values)) = (s.get("items"), v.as_array()) {
        for value in values {
            validate_inner(items, value)?;
        }
    }
    Ok(())
}

pub fn validate(s: &Value, v: &Value) -> crate::Result<()> {
    validate_inner(s, v).map_err(|e| crate::Error::new(crate::ErrorKind::InvalidValue, e))
}
pub fn decode<T: DeserializeOwned>(s: &Value, v: Value) -> crate::Result<T> {
    validate(s, &v)?;
    serde_json::from_value(v)
        .map_err(|e| crate::Error::new(crate::ErrorKind::InvalidValue, e.to_string()))
}

// Compare finite JSON numbers exactly in their serialized decimal representation,
// avoiding integer-to-f64 loss for const/enum (notably above 2^53).
fn number_key(n: &serde_json::Number) -> (bool, String, i32) {
    let text = n.to_string();
    let (negative, text) = text
        .strip_prefix('-')
        .map_or((false, text.as_str()), |s| (true, s));
    let (mantissa, exponent) = text.split_once(['e', 'E']).map_or((text, 0), |(m, e)| {
        (m, e.parse::<i32>().expect("JSON exponent"))
    });
    let fraction = mantissa.split_once('.').map_or(0, |(_, f)| f.len() as i32);
    let mut digits = mantissa
        .replace('.', "")
        .trim_start_matches('0')
        .to_string();
    if digits.is_empty() {
        return (false, "0".into(), 0);
    }
    let mut scale = exponent - fraction;
    while digits.ends_with('0') {
        digits.pop();
        scale += 1;
    }
    (negative, digits, scale)
}
fn equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => number_key(a) == number_key(b),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len() && a.iter().all(|(k, a)| b.get(k).is_some_and(|b| equal(a, b)))
        }
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keyword_contracts_are_checked_without_serde() {
        let cases = [
            (
                json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"],"additionalProperties":false}),
                json!({"name":"ok"}),
                json!({"name":"ok","extra":1}),
            ),
            (
                json!({"type":"array","items":{"type":"boolean"}}),
                json!([true, false]),
                json!([true, 0]),
            ),
            (json!({"type":"null"}), json!(null), json!("null")),
            (json!({"type":"number"}), json!(1.5), json!("1.5")),
            (
                json!({"type":"integer","format":"int64"}),
                json!(i64::MAX),
                json!(u64::MAX),
            ),
            (json!({"enum":["a","b"]}), json!("a"), json!("c")),
            (json!({"const":"a"}), json!("a"), json!("b")),
            (
                json!({"anyOf":[{"type":"string"},{"type":"boolean"}]}),
                json!(true),
                json!(2),
            ),
            (
                json!({"oneOf":[{"type":"integer"},{"type":"number"}]}),
                json!(2.5),
                json!(2),
            ),
        ];
        for (schema, good, bad) in cases {
            supported(&schema).unwrap();
            validate(&schema, &good).unwrap();
            assert!(validate(&schema, &bad).is_err(), "{schema}: {bad}");
        }
        assert!(validate(&json!({"type":"object","required":["x"]}), &json!({})).is_err());
        assert!(validate(&json!({"type":"integer"}), &json!(3.0)).is_err());
    }
    #[test]
    fn numeric_constants_do_not_round_large_integers() {
        assert!(validate(&json!({"const":1}), &json!(1.0)).is_ok());
        assert!(validate(&json!({"enum":[{"x":0}]}), &json!({"x":-0.0})).is_ok());
        assert!(
            validate(
                &json!({"const":9007199254740993u64}),
                &json!(9007199254740992.0)
            )
            .is_err()
        );
        assert!(
            validate(
                &json!({"const":1000}),
                &serde_json::from_str("1e3").unwrap()
            )
            .is_ok()
        );
    }
    #[test]
    fn unsupported_schemas_are_located() {
        for (schema, path) in [
            (
                json!({"$schema":"http://json-schema.org/draft-07/schema#"}),
                "/$schema",
            ),
            (
                json!({"properties":{"nested":{"$ref":"#/$defs/X"}}}),
                "/properties/nested/$ref",
            ),
            (json!({"$defs":{}}), "/$defs"),
            (json!({"items":{"minLength":2}}), "/items/minLength"),
            (json!({"type":"string","format":"email"}), "/format"),
            (json!({"minimum":0}), "/minimum"),
            (json!({"oneOf":[]}), "/oneOf"),
        ] {
            assert!(supported(&schema).unwrap_err().contains(path));
        }
    }
}
