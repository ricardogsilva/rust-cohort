// declare modules - without the `pub` modifier, these are private
mod error;
mod parser;
mod tokenizer;
mod value;

// re-export types and functions (note the `pub` modifier) to make them easier to call by third-party code
pub use error::JsonError;
pub use parser::JsonParser;
pub use tokenizer::{Token, Tokenizer};
pub use value::JsonValue;

// convenience type alias - this lets us refer to Result<JsonValue> instead of Result<JsonValue, JsonError>
pub type Result<T> = std::result::Result<T, JsonError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration() -> Result<()> {
        // test whole parsing pipeline

        let test_cases = vec![
            ("null", JsonValue::Null),
            ("true", JsonValue::Boolean(true)),
            ("false", JsonValue::Boolean(false)),
            ("42", JsonValue::Number(42.0)),
            ("0", JsonValue::Number(0.0)),
            ("-10", JsonValue::Number(-10.0)),
            (r#""hello""#, JsonValue::String("hello".to_string())),
        ];

        for (input, expected) in test_cases {
            let mut parser = JsonParser::new(input)?;
            let result = parser.parse()?;
            assert_eq!(result, expected, "Failed for input: {}", input);
        }

        Ok(())
    }

    #[test]
    fn test_error_propagation() {
        let parser = JsonParser::new("@invalid@");
        assert!(parser.is_err());

        match parser {
            Err(JsonError::UnexpectedToken {
                expected,
                found,
                position,
            }) => {
                assert_eq!(expected, "valid JSON token");
                assert_eq!(found, "@");
                assert_eq!(position, 0);
            }
            _ => panic!("Expected UnexpectedToken error"),
        }
    }
}
