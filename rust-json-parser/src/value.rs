use std::collections::HashMap;
// use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Boolean(bool),
    Null,
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

// impl fmt::Display for JsonValue {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             JsonValue::Boolean(v) => write!(f, "{v}"),
//             JsonValue::Null => write!(f, "null"),
//             JsonValue::Number(v) => write!(f, "{v}"),
//             JsonValue::String(v) => write!(f, "{v}"),
//             JsonValue::Array(v) => write!(f, "{v}"),
//             JsonValue::Object(v) => write!(f, "{v}"),
//         }
//     }
// }

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(owned_string) => Some(owned_string.as_str()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(num_val) => Some(*num_val),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Boolean(bool_val) => Some(*bool_val),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(v) => Some(v),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.as_object()?.get(key)
    }

    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        self.as_array()?.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_value_equality() {
        assert_eq!(JsonValue::Null, JsonValue::Null);
        assert_eq!(JsonValue::Boolean(true), JsonValue::Boolean(true));
        assert_eq!(JsonValue::Number(42.0), JsonValue::Number(42.0));
        assert_eq!(
            JsonValue::String("test".to_string()),
            JsonValue::String("test".to_string())
        );
        assert_ne!(JsonValue::Null, JsonValue::Boolean(false));
        assert_ne!(JsonValue::Number(1.0), JsonValue::Number(2.0));
    }

    #[test]
    fn test_json_value_creation() {
        let null_val = JsonValue::Null;
        let bool_val = JsonValue::Boolean(true);
        let num_val = JsonValue::Number(42.5);
        let str_val = JsonValue::String("hello".to_string());

        assert!(null_val.is_null());
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(num_val.as_f64(), Some(42.5));
        assert_eq!(str_val.as_str(), Some("hello"));
    }

    #[test]
    fn test_json_value_accessors() {
        let value = JsonValue::String("test".to_string());
        assert_eq!(value.as_str(), Some("test"));
        assert_eq!(value.as_f64(), None);
        assert_eq!(value.as_bool(), None);
        assert!(!value.is_null());

        let value = JsonValue::Number(42.0);
        assert_eq!(value.as_f64(), Some(42.0));
        assert_eq!(value.as_str(), None);

        let value = JsonValue::Boolean(true);
        assert_eq!(value.as_bool(), Some(true));

        let value = JsonValue::Null;
        assert!(value.is_null());
    }
}
