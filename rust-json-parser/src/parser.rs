use crate::error::JsonError;
use crate::tokenizer::{Token, tokenize};
use crate::value::JsonValue;

// this is a type alias and we define it just for convenience
// T is a generic type, which we will particularize later, at
// call sites. JsonError is a concrete type, so we don't have
// to keep using it at call sites, the compiler already knows
// it
type Result<T> = std::result::Result<T, JsonError>;

pub fn parse_json(json_text: &str) -> Result<JsonValue> {
    let tokens = tokenize(json_text)?;
    if tokens.is_empty() {
        return Err(
            JsonError::UnexpectedEndOfInput { 
                expected: "JSON value".to_string(), 
                position: 0 
            }
        )
    }
    match &tokens[0] {
        Token::Boolean(bool_value) => Ok(JsonValue::Boolean(*bool_value)),
        Token::Number(num_val) => Ok(JsonValue::Number(*num_val)),
        Token::Null => Ok(JsonValue::Null),
        Token::String(string_val) => {
            // - Is this copying the token's string onto the value's?
            // How could we make it move the string instead?
            // The module hints have this as `Ok(JsonValue::String(string_val.clone()))`
            Ok(JsonValue::String(string_val.to_string()))
        },
        _ => {
            Err(
                JsonError::UnexpectedToken { 
                    expected: "Only boolean, number, string and null are supported for now".to_string(), 
                    found: (format!("{:?}", tokens[0])), 
                    position: 0
                }
            )
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FIXME: understand why we are defining this again here?
    type Result<T> = std::result::Result<T, JsonError>;

    #[test]
    // FIXME: understand why tests now return something?
    fn test_parse_string() -> Result<()>{
        // using the ? operator here means the test will immediately fail 
        // if result is an error
        let result = parse_json(r#""hello world""#)?;
        assert_eq!(result, JsonValue::String("hello world".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_number() -> Result<()> {
        let result = parse_json("42.5")?;
        assert_eq!(result, JsonValue::Number(42.5));
        
        let result = parse_json("0")?;
        assert_eq!(result, JsonValue::Number(0.0));
        
        let result = parse_json("-10")?;
        assert_eq!(result, JsonValue::Number(-10.0));
        
        Ok(())
    }

    #[test]
    fn test_parse_boolean() -> Result<()> {
        let result = parse_json("true")?;
        assert_eq!(result, JsonValue::Boolean(true));

        let result = parse_json("false")?;
        assert_eq!(result, JsonValue::Boolean(false));

        Ok(())
    }

    #[test]
    fn test_parse_null() -> Result<()> {
        let result = parse_json("null")?;
        assert_eq!(result, JsonValue::Null);
        Ok(())
    }

    #[test]
    fn test_parse_error_empty() {
        let result = parse_json("");
        assert!(result.is_err());

        match result {
            Err(JsonError::UnexpectedEndOfInput { expected, position }) => {
                assert_eq!(expected, "JSON value");
                assert_eq!(position, 0);
            },
            _ => panic!("Expected UnexpectedEndOfInput error"),
        }
    }

    #[test]
    fn test_parse_error_invalid_token() {
        let result = parse_json("@");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_whitespace() -> Result<()> {
        let result = parse_json("  42  ")?;
        assert_eq!(result, JsonValue::Number(42.0));

        let result = parse_json("\n\ttrue\n")?;
        assert_eq!(result, JsonValue::Boolean(true));

        Ok(())
    }

    #[test]
    fn test_result_pattern_matching() {
        let result = parse_json("42");
        match result {
            Ok(JsonValue::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected successful number parse"),
        }

        let result = parse_json("@invalid@");
        match result {
            Err(JsonError::UnexpectedToken { .. }) => {},
            _ => panic!("Expected UnexpectedTokenError error"),
        }
    }
}