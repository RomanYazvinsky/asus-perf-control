//! Lenient JSON readers. Each invalid value is reported to `issues` and replaced by a
//! default, exactly like the Dart settings readers they replace.

use serde_json::{Map, Value};

pub type JsonMap = Map<String, Value>;

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "Null",
        Value::Bool(_) => "bool",
        Value::Number(n) if n.is_i64() || n.is_u64() => "int",
        Value::Number(_) => "double",
        Value::String(_) => "String",
        Value::Array(_) => "List<dynamic>",
        Value::Object(_) => "_Map<String, dynamic>",
    }
}

fn as_int(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_i64(),
        _ => None,
    }
}

pub fn read_map(json: &JsonMap, key: &str, issues: &mut Vec<String>) -> JsonMap {
    match json.get(key) {
        None | Some(Value::Null) => JsonMap::new(),
        Some(Value::Object(map)) => map.clone(),
        Some(other) => {
            issues.push(format!("{key} invalidType actual={}", type_name(other)));
            JsonMap::new()
        }
    }
}

pub fn read_optional_map(
    json: &JsonMap,
    key: &str,
    path: &str,
    issues: &mut Vec<String>,
) -> Option<JsonMap> {
    match json.get(key) {
        None | Some(Value::Null) => None,
        Some(Value::Object(map)) => Some(map.clone()),
        Some(other) => {
            issues.push(format!("{path} invalidType actual={}", type_name(other)));
            None
        }
    }
}

pub fn read_int(json: &JsonMap, key: &str, path: &str, default: i64, issues: &mut Vec<String>) -> i64 {
    match json.get(key) {
        None | Some(Value::Null) => default,
        Some(value) => match as_int(value) {
            Some(v) => v,
            None => {
                issues.push(format!("{path} invalidType actual={}", type_name(value)));
                default
            }
        },
    }
}

pub fn read_ranged_int(
    json: &JsonMap,
    key: &str,
    path: &str,
    (min, max): (i64, i64),
    default: i64,
    issues: &mut Vec<String>,
) -> i64 {
    read_optional_ranged_int(json, key, path, (min, max), issues).unwrap_or(default)
}

pub fn read_optional_ranged_int(
    json: &JsonMap,
    key: &str,
    path: &str,
    (min, max): (i64, i64),
    issues: &mut Vec<String>,
) -> Option<i64> {
    let value = match json.get(key) {
        None | Some(Value::Null) => return None,
        Some(value) => value,
    };
    let Some(v) = as_int(value) else {
        issues.push(format!("{path} invalidType actual={}", type_name(value)));
        return None;
    };
    if v < min || v > max {
        issues.push(format!("{path} outOfRange actual={v} expected={min}..{max}"));
        return None;
    }
    Some(v)
}

pub fn read_bool(json: &JsonMap, key: &str, path: &str, default: bool, issues: &mut Vec<String>) -> bool {
    match json.get(key) {
        None | Some(Value::Null) => default,
        Some(Value::Bool(b)) => *b,
        Some(other) => {
            issues.push(format!("{path} invalidType actual={}", type_name(other)));
            default
        }
    }
}

pub fn read_optional_string(
    json: &JsonMap,
    key: &str,
    path: &str,
    issues: &mut Vec<String>,
) -> Option<String> {
    match json.get(key) {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(_) => {
            issues.push(format!("{path} invalidValue"));
            None
        }
    }
}

/// Required non-empty string; reports `<key> invalidValue` and returns "" when missing.
pub fn read_string(json: &JsonMap, key: &str, issues: &mut Vec<String>) -> String {
    match json.get(key) {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            issues.push(format!("{key} invalidValue"));
            String::new()
        }
    }
}

pub fn read_string_map(
    json: &JsonMap,
    key: &str,
    path: &str,
    issues: &mut Vec<String>,
) -> Vec<(String, String)> {
    let map = match json.get(key) {
        None | Some(Value::Null) => return Vec::new(),
        Some(Value::Object(map)) => map,
        Some(other) => {
            issues.push(format!("{path} invalidType actual={}", type_name(other)));
            return Vec::new();
        }
    };
    let mut result = Vec::new();
    for (entry_key, value) in map {
        match value {
            Value::String(s) => result.push((entry_key.clone(), s.clone())),
            _ => issues.push(format!("{path}.{entry_key} invalidType")),
        }
    }
    result
}
