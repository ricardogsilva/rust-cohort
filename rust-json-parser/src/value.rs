use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Boolean(bool),
    Null,
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

fn display_json_string(original: &str) -> String {
    let mut result = String::new();
    for ch in original.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\u{0008}' => result.push_str("\\b"),
            '\u{000C}' => result.push_str("\\f"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '/' => result.push_str("\\/"),
            // The JSON RFC's section 7 on Strings mentions that the first unicode
            // code points not already covered above up until 0x20 must always be escaped
            // - I found it a bit thick to understand, but basically its grammar defines
            // the 'unescaped' term and being any codepoint that is not
            // 0x20, 0x21, 0x23-0x5B and 0x5D-0x10FFFF to require explicit escaping
            c if (c as u32) < 0x20 => result.push_str(&format!("\\u{:04x}", c as u32)),
            _ => result.push(ch),
        }
    }
    result
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Boolean(v) => write!(f, "{v}"),
            JsonValue::Null => write!(f, "null"),
            JsonValue::Number(v) => write!(f, "{v}"),
            JsonValue::String(v) => write!(f, "\"{}\"", display_json_string(v)),
            JsonValue::Array(v) => {
                write!(f, "[")?;
                for (index, item) in v.iter().enumerate() {
                    write!(f, "{item}")?;
                    if index < v.len() - 1 {
                        write!(f, ",")?;
                    }
                }
                write!(f, "]")
            }
            JsonValue::Object(v) => {
                write!(f, "{{")?;
                for (index, (key, value)) in v.iter().enumerate() {
                    write!(f, "\"{}\": {value}", display_json_string(key))?;
                    if index < v.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn pretty_print(val: &JsonValue, depth: Option<usize>, indent: Option<usize>) -> String {
        let chosen_depth = depth.unwrap_or(0);
        let chosen_indent = indent.unwrap_or(2);
        match val {
            JsonValue::Null => String::from("null"),
            JsonValue::Boolean(v) => v.to_string(),
            JsonValue::Number(v) => v.to_string(),
            JsonValue::String(v) => format!("\"{}\"", display_json_string(&v)),
            JsonValue::Array(v) => JsonValue::pretty_print_array(v, chosen_depth, chosen_indent),
            JsonValue::Object(v) => JsonValue::pretty_print_object(v, chosen_depth, chosen_indent),
        }
    }

    fn pretty_print_object(obj: &HashMap<String, JsonValue>, depth: usize, indent: usize) -> String {
        let padding = JsonValue::get_pretty_print_padding(depth, indent);
        let inner_padding = JsonValue::get_pretty_print_padding(depth + 1, indent);
        let mut result = String::new();
        if obj.is_empty() {
            result.push_str(&format!("{}{{}}", padding));
            return result;
        }
        result.push_str(&format!("{}{{\n", padding));
        for (index, (key, value)) in obj.iter().enumerate() {
            result.push_str(&format!("{}\"{}\"", inner_padding, display_json_string(&key)));
            result.push_str(": ");
            result.push_str(&JsonValue::pretty_print(value, Some(depth + 1), Some(indent)));
            if index < obj.len() - 1 {
                result.push_str(",\n")
            } else {
                result.push('\n');
            }
        }
        result.push('}');
        result
    }

    fn pretty_print_array(arr: &Vec<JsonValue>, depth: usize, indent: usize) -> String {
        let inner_padding = JsonValue::get_pretty_print_padding(depth + 1, indent);
        let mut result = String::new();
        if arr.is_empty() {
            return "[]".to_string();
        }
        result.push_str("[\n");
        for (index, item) in arr.iter().enumerate() {
            let mut pretty_item = format!("{}{}", inner_padding, JsonValue::pretty_print(item, Some(depth + 1), Some(indent)));
            if index < arr.len() - 1{
                pretty_item.push_str(",\n");
            } else {
                pretty_item.push('\n');
            }
            result.push_str(&pretty_item);
        }

        if depth > 0 {
            let padding = JsonValue::get_pretty_print_padding(depth, indent);
            result.push_str(&format!("{}]", padding));
        } else {
            result.push_str("]");
        }
        result
    }
    
    fn get_pretty_print_padding(depth: usize, indent: usize) -> String {
        format!("{}", " ".repeat(depth * indent))
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
            JsonValue::Array(v) => Some(&v),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(v) => Some(&v),
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
    use crate::parser::JsonParser;
    
    #[test]
    fn test_pretty_print_array() {
        let mut parser = JsonParser::new(r#"["sleep", "eat", "run"]"#).unwrap();
        let parsed = parser.parse().unwrap();
        println!("{}", JsonValue::pretty_print(&parsed, None, None));
        let expected = String::from("[\n  \"sleep\",\n  \"eat\",\n  \"run\"\n]");
        println!("{}", expected);
        assert_eq!(JsonValue::pretty_print(&parsed, None, None), expected);
    }

    #[test]
    fn test_pretty_print_object() {
        let mut parser = JsonParser::new(r#"{"name": "Alice", "age": 30, "likes_peaches": true, "pet": null, "favorite_things": ["climb", "eat"]}"#).unwrap();
        let parsed = parser.parse().unwrap();
        println!("{}", JsonValue::pretty_print(&parsed, None, None));
        let expected = String::from("{\n  \"name\": \"Alice\",\n  \"age\": 30.0,\n  \"likes_peaches\": true,\n  \"pet\": null,\n  \"favorite_things\": [\n    \"climb\",\n    \"eat\"\n  ]\n}");
        println!("{}", expected);
        assert_eq!(JsonValue::pretty_print(&parsed, None, None), expected);
    }

    #[test]
    fn test_display_primitives() {
        assert_eq!(JsonValue::Null.to_string(), "null");
        assert_eq!(JsonValue::Boolean(true).to_string(), "true");
        assert_eq!(JsonValue::Boolean(false).to_string(), "false");
        assert_eq!(JsonValue::Number(42.0).to_string(), "42");
        assert_eq!(JsonValue::Number(3.14).to_string(), "3.14");
        assert_eq!(
            JsonValue::String("hello".to_string()).to_string(),
            "\"hello\""
        );
    }

    #[test]
    fn test_display_array() {
        let value = JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]);
        assert_eq!(value.to_string(), "[1,2]");
    }

    #[test]
    fn test_display_empty_containers() {
        assert_eq!(JsonValue::Array(vec![]).to_string(), "[]");
        assert_eq!(JsonValue::Object(HashMap::new()).to_string(), "{}");
    }

    #[test]
    fn test_display_escape_string() {
        let value = JsonValue::String("hello\nworld".to_string());
        assert_eq!(value.to_string(), "\"hello\\nworld\"");
    }

    #[test]
    fn test_display_escape_quotes() {
        let value = JsonValue::String("say \"hi\"".to_string());
        assert_eq!(value.to_string(), "\"say \\\"hi\\\"\"");
    }

    #[test]
    fn test_display_nested() {
        let mut parser = JsonParser::new(r#"{"arr": [1, 2]}"#).unwrap();
        let value = parser.parse().unwrap();
        let output = value.to_string();
        // Object key order may vary, so check components
        assert!(output.contains("\"arr\""));
        assert!(output.contains("[1,2]"));
    }

    #[test]
    fn test_display_nested_array() {
        let value = JsonValue::Array(vec![JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
        ])]);
        assert_eq!(value.to_string(), "[[1,2]]");
    }

    // tests carried over from week 3

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
