use crate::error::JsonError;
use crate::tokenizer::{Token, Tokenizer};
use crate::value::JsonValue;

// this is a type alias and we define it just for convenience
// T is a generic type, which we will particularize later, at
// call sites. JsonError is a concrete type, so we don't have
// to keep using it at call sites, the compiler already knows
// it
type Result<T> = std::result::Result<T, JsonError>;


#[derive(Debug, Clone, PartialEq)]
pub struct JsonParser {
    tokens: Vec<Token>,
    position: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Result<Self> {
        let mut tokenizer = Tokenizer::new(input);
        match tokenizer.tokenize() {
            Ok(tokens) => {
                match tokens.is_empty() {
                    true => {
                        Err(
                            JsonError::UnexpectedEndOfInput { 
                                expected: "JSON value".to_string(), 
                                position: 0 
                            }
                        )
                    }
                    false => {
                        Ok(
                            JsonParser {
                                tokens: tokens,
                                position: 0,
                            }
                        )
                    }
                }
            }
            Err(e) => Err(e)
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue> {
        match &self.tokens[0] {
            Token::Boolean(bool_value) => Ok(JsonValue::Boolean(*bool_value)),
            Token::Number(num_val) => Ok(JsonValue::Number(*num_val)),
            Token::Null => Ok(JsonValue::Null),
            Token::String(string_val) => {
                Ok(JsonValue::String(string_val.clone()))
            }
            _ => Err(JsonError::UnexpectedToken {
                expected: "Only boolean, number, string and null are supported for now".to_string(),
                found: (format!("{:?}", self.tokens[0])),
                position: 0,
            }),
            
        }
    }

    fn advance(&mut self) -> Option<Token> {
        // move to next token
        match self.is_at_end() {
            true => None,
            false => {
                self.position += 1;
                Some(self.tokens[self.position].clone())
            }
        }
    }

    fn is_at_end(&self) -> bool {
        // check if consumed all tokens
        self.position >= self.tokens.len()
    }
}

pub fn parse_json(json_text: &str) -> Result<JsonValue> {
    let mut tokenizer = Tokenizer::new(json_text);
    let tokens = tokenizer.tokenize()?;
    if tokens.is_empty() {
        return Err(JsonError::UnexpectedEndOfInput {
            expected: "JSON value".to_string(),
            position: 0,
        });
    }
    match &tokens[0] {
        Token::Boolean(bool_value) => Ok(JsonValue::Boolean(*bool_value)),
        Token::Number(num_val) => Ok(JsonValue::Number(*num_val)),
        Token::Null => Ok(JsonValue::Null),
        Token::String(string_val) => {
            // - Is this copying the token's string onto the value's?
            // How could we make it move the string instead?
            Ok(JsonValue::String(string_val.clone()))
        }
        _ => Err(JsonError::UnexpectedToken {
            expected: "Only boolean, number, string and null are supported for now".to_string(),
            found: (format!("{:?}", tokens[0])),
            position: 0,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result<T> = std::result::Result<T, JsonError>;

    #[test]
    fn test_parser_creation() {
        let parser = JsonParser::new("42");
        assert!(parser.is_ok());
    }
    
    #[test]
    fn test_parser_creation_tokenize_error() {
        let parser = JsonParser::new(r#""\q""#);
        assert!(parser.is_err());
    }
    
    #[test]
    fn test_parse_number() -> Result<()> {
        let mut parser = JsonParser::new("42.5").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Number(42.5));

        let mut parser = JsonParser::new("0")?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::Number(0.0));

        let mut parser = JsonParser::new("-10")?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::Number(-10.0));

        Ok(())
    }

    // Tests carried over from week2 (adapted to work with JsonParser struct)

    #[test]
    fn test_parse_string() -> Result<()> {
        // using the ? operator here means the test will immediately fail
        // if result is an error
        let result = parse_json(r#""hello world""#)?;
        assert_eq!(result, JsonValue::String("hello world".to_string()));
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
            }
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
            Err(JsonError::UnexpectedToken { .. }) => {}
            _ => panic!("Expected UnexpectedTokenError error"),
        }
    }
}
