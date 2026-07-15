use std::collections::HashMap;

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
            Ok(tokens) => match tokens.is_empty() {
                true => Err(JsonError::UnexpectedEndOfInput {
                    expected: "JSON value".to_string(),
                    position: 0,
                }),
                false => Ok(JsonParser {
                    tokens,
                    position: 0,
                }),
            },
            Err(e) => Err(e),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue> {
        match self.is_at_end() {
            true => Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON token".to_string(),
                position: self.position,
            }),
            false => match self.advance() {
                Some(token) => {
                    match token {
                        Token::Boolean(bool_value) => Ok(JsonValue::Boolean(bool_value)),
                        Token::Number(num_val) => Ok(JsonValue::Number(num_val)),
                        Token::Null => Ok(JsonValue::Null),
                        Token::String(string_val) => Ok(JsonValue::String(string_val.clone())),
                        Token::LeftBracket => self.parse_array(),
                        Token::LeftBrace => self.parse_object(),
                        _ => Err(JsonError::UnexpectedToken {
                            expected: "Either a string, number, true, false, null, start of array or start of object".to_string(),
                            found: (format!("{:?}", token)),
                            position: self.position,
                        }),
                    }
                },
                None => Err(JsonError::UnexpectedEndOfInput { expected: "JSON token".to_string(), position: self.position }),
            }
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        let mut array_contents: Vec<JsonValue> = Vec::new();

        // this variable is used as a flag in order to know whether we expect
        // the next token to be a Comma or not. In a JSON array, commas cannot
        // appear as the first or last token of the array and they must occur between other
        // tokens.
        // This also means that the array can be ended whenever we are expecting a comma
        let mut expecting_comma = false;

        let mut parsing_array = true;
        while let Some(next_token) = self.peek() {
            match next_token {
                Token::Comma if expecting_comma => {
                    expecting_comma = false;
                    self.advance();
                }
                Token::Comma if !expecting_comma => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "value token".to_string(),
                        found: "comma".to_string(),
                        position: self.position,
                    });
                }
                Token::RightBracket if array_contents.is_empty() || expecting_comma => {
                    parsing_array = false;
                    self.advance();
                    break;
                }
                Token::RightBracket if !expecting_comma => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "End of array".to_string(),
                        found: "Dangling comma".to_string(),
                        position: self.position - 1,
                    });
                }
                something_else => match expecting_comma {
                    true => {
                        return Err(JsonError::UnexpectedToken {
                            expected: "comma".to_string(),
                            found: format!("{:?}", something_else),
                            position: self.position,
                        });
                    }
                    false => {
                        array_contents.push(self.parse()?);
                        expecting_comma = true;
                    }
                },
            }
        }
        match parsing_array {
            true => Err(JsonError::UnexpectedEndOfInput {
                expected: "End of array".to_string(),
                position: self.position,
            }),
            false => Ok(JsonValue::Array(array_contents)),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        let mut expecting_comma = false;
        let mut object_contents: HashMap<String, JsonValue> = HashMap::new();
        let mut parsing_object = true;
        while let Some(current) = self.advance() {
            match current {
                Token::Comma if expecting_comma => expecting_comma = false,
                Token::Comma if !expecting_comma => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "string or end of object".to_string(),
                        found: "comma".to_string(),
                        position: self.position,
                    });
                }
                Token::RightBrace if object_contents.is_empty() || expecting_comma => {
                    parsing_object = false;
                    break;
                }
                Token::RightBrace if !expecting_comma => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "string".to_string(),
                        found: "end of object".to_string(),
                        position: self.position,
                    });
                }
                Token::String(v) if expecting_comma => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "comma".to_string(),
                        found: format!("{:?}", v),
                        position: self.position,
                    });
                }
                Token::String(v) if !expecting_comma => {
                    let key = v.clone();
                    match self.advance() {
                        Some(maybe_colon) => match maybe_colon {
                            Token::Colon => {
                                let value = self.parse()?;
                                object_contents.insert(key, value);
                                expecting_comma = true;
                            }
                            _ => {
                                return Err(JsonError::UnexpectedToken {
                                    expected: "colon".to_string(),
                                    found: format!("{:?}", maybe_colon),
                                    position: self.position,
                                });
                            }
                        },
                        None => {
                            return Err(JsonError::UnexpectedEndOfInput {
                                expected: "colon".to_string(),
                                position: self.position,
                            });
                        }
                    }
                }
                _ => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "either a comma, right brace or a string".to_string(),
                        found: format!("{:?}", current),
                        position: self.position,
                    });
                }
            }
        }
        match parsing_object {
            true => Err(JsonError::UnexpectedEndOfInput {
                expected: "End of object".to_string(),
                position: self.position,
            }),
            false => Ok(JsonValue::Object(object_contents)),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        // return current and move to next token
        match self.is_at_end() {
            true => None,
            false => {
                self.position += 1;
                Some(self.tokens[self.position - 1].clone())
            }
        }
    }

    fn peek(&self) -> Option<&Token> {
        // return current and stay put
        match self.is_at_end() {
            true => None,
            false => match self.position {
                n if n <= self.tokens.len() => Some(&self.tokens[self.position]),
                _ => None,
            },
        }
    }

    fn is_at_end(&self) -> bool {
        // check if consumed all tokens
        self.position >= self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    type Result<T> = std::result::Result<T, JsonError>;

    #[test]
    fn test_parse_empty_array() {
        let mut parser = JsonParser::new("[]").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Array(vec![]))
    }

    #[test]
    fn test_parse_array_single() {
        let mut parser = JsonParser::new("[1]").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Array(vec![JsonValue::Number(1.0)]))
    }

    #[test]
    fn test_parse_array_multiple() {
        let mut parser = JsonParser::new("[1, 2, 3]").unwrap();
        let value = parser.parse().unwrap();
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(value, expected)
    }

    #[test]
    fn test_parse_array_mixed_types() {
        let mut parser = JsonParser::new(r#"[1, "two", true, null]"#).unwrap();
        let value = parser.parse().unwrap();
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::String("two".to_string()),
            JsonValue::Boolean(true),
            JsonValue::Null,
        ]);
        assert_eq!(value, expected)
    }

    #[test]
    fn test_array_accessor() {
        let mut parser = JsonParser::new("[1, 2, 3]").unwrap();
        let parsed = parser.parse().unwrap();
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_array_get_index() {
        let mut parser = JsonParser::new("[10, 20, 30]").unwrap();
        let parsed = parser.parse().unwrap();
        assert_eq!(parsed.get_index(1), Some(&JsonValue::Number(20.0)));
        assert_eq!(parsed.get_index(5), None);
    }

    #[test]
    fn test_parse_empty_object() {
        let mut parser = JsonParser::new("{}").unwrap();
        let parsed = parser.parse().unwrap();
        assert_eq!(parsed, JsonValue::Object(HashMap::new()));
    }

    #[test]
    fn test_parse_object_single_key() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#).unwrap();
        let parsed = parser.parse().unwrap();
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parsed, JsonValue::Object(expected));
    }

    #[test]
    fn test_parse_object_multiple_keys() {
        let mut parser = JsonParser::new(r#"{"name": "Alice", "age": 30}"#).unwrap();
        let parsed = parser.parse().unwrap();
        if let JsonValue::Object(obj) = parsed {
            assert_eq!(
                obj.get("name"),
                Some(&JsonValue::String("Alice".to_string()))
            );
            assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
        } else {
            panic!("Expected object")
        }
    }

    #[test]
    fn test_object_accessor() {
        let mut parser = JsonParser::new(r#"{"name": "test"}"#).unwrap();
        let value = parser.parse().unwrap();
        let obj = value.as_object().unwrap();
        assert_eq!(obj.len(), 1);
    }

    #[test]
    fn test_object_get() {
        let mut parser = JsonParser::new(r#"{"name": "Alice", "age": 30}"#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(
            value.get("name"),
            Some(&JsonValue::String("Alice".to_string()))
        );
        assert_eq!(value.get("missing"), None);
    }

    #[test]
    fn test_parse_nested_arrays() {
        let mut parser = JsonParser::new("[[1, 2], [3, 4]]").unwrap();
        let value = parser.parse().unwrap();
        let expected = JsonValue::Array(vec![
            JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
            JsonValue::Array(vec![JsonValue::Number(3.0), JsonValue::Number(4.0)]),
        ]);
        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_deeply_nested() {
        let mut parser = JsonParser::new("[[[1]]]").unwrap();
        let value = parser.parse().unwrap();
        let expected = JsonValue::Array(vec![JsonValue::Array(vec![JsonValue::Array(vec![
            JsonValue::Number(1.0),
        ])])]);
        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_nested_object() {
        let mut parser = JsonParser::new(r#"{"outer": {"inner": 1}}"#).unwrap();
        let value = parser.parse().unwrap();
        if let JsonValue::Object(outer) = value {
            if let Some(JsonValue::Object(inner)) = outer.get("outer") {
                assert_eq!(inner.get("inner"), Some(&JsonValue::Number(1.0)));
            } else {
                panic!("Expected nested object");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array_in_object() {
        let mut parser = JsonParser::new(r#"{"items": [1, 2, 3]}"#).unwrap();
        let value = parser.parse().unwrap();
        if let JsonValue::Object(obj) = value {
            if let Some(JsonValue::Array(arr)) = obj.get("items") {
                assert_eq!(arr.len(), 3);
            } else {
                panic!("Expected array");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_object_in_array() {
        let mut parser = JsonParser::new(r#"[{"a": 1}, {"b": 2}]"#).unwrap();
        let value = parser.parse().unwrap();
        if let JsonValue::Array(arr) = value {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_error_unclosed_array() {
        let mut parser = JsonParser::new("[1, 2").unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unclosed_object() {
        let mut parser = JsonParser::new(r#"{"key": 1"#).unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_trailing_comma_array() {
        let mut parser = JsonParser::new("[1, 2,]").unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_trailing_comma_object() {
        let mut parser = JsonParser::new(r#"{"a": 1,}"#).unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_colon() {
        let mut parser = JsonParser::new(r#"{"key" 1}"#).unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_invalid_key() {
        let mut parser = JsonParser::new(r#"{123: "value"}"#).unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_comma_array() {
        let mut parser = JsonParser::new("[1 2 3]").unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_comma_object() {
        let mut parser = JsonParser::new(r#"{"a": 1 "b": 2}"#).unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }

    // tests carried over from week3

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

    #[test]
    fn test_parse_boolean() -> Result<()> {
        let mut parser = JsonParser::new("true")?;
        let result = parser.parse()?;
        assert_eq!(result, JsonValue::Boolean(true));

        let mut parser = JsonParser::new("false")?;
        let result = parser.parse()?;
        assert_eq!(result, JsonValue::Boolean(false));

        Ok(())
    }

    #[test]
    fn test_parse_string() -> Result<()> {
        // using the ? operator here means the test will immediately fail
        // if result is an error
        let mut parser = JsonParser::new(r#""hello world""#)?;
        let result = parser.parse()?;
        assert_eq!(result, JsonValue::String("hello world".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_null() -> Result<()> {
        let mut parser = JsonParser::new("null")?;
        let result = parser.parse()?;
        assert_eq!(result, JsonValue::Null);
        Ok(())
    }

    #[test]
    fn test_parse_negative_number() {
        let mut parser = JsonParser::new("-3.14").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_error_empty() {
        let parser = JsonParser::new("");
        assert!(parser.is_err());

        match parser {
            Err(JsonError::UnexpectedEndOfInput { expected, position }) => {
                assert_eq!(expected, "JSON value");
                assert_eq!(position, 0);
            }
            _ => panic!("Expected UnexpectedEndOfInput error"),
        }
    }

    #[test]
    fn test_parse_whitespace_only() {
        let parser = JsonParser::new("   ");
        assert!(parser.is_err());

        match parser {
            Err(JsonError::UnexpectedEndOfInput { expected, position }) => {
                assert_eq!(expected, "JSON value");
                assert_eq!(position, 0);
            }
            _ => panic!("Expected UnexpectedEndOfInput error"),
        }
    }

    #[test]
    fn test_parse_string_with_newline() {
        let mut parser = JsonParser::new(r#""hello\nworld""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("hello\nworld".to_string()));
    }

    #[test]
    fn test_parse_string_with_unicode() {
        let mut parser = JsonParser::new(r#""\u0048\u0065\u006c\u006c\u006f""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("Hello".to_string()));
    }

    #[test]
    fn test_parse_complex_escapes() {
        let mut parser = JsonParser::new(r#""line1\nline2\t\"quoted\"\u0021""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(
            value,
            JsonValue::String("line1\nline2\t\"quoted\"!".to_string())
        );
    }

    #[test]
    fn test_parse_string_with_tab() {
        let mut parser = JsonParser::new(r#""col1\tcol2""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("col1\tcol2".to_string()));
    }

    #[test]
    fn test_parse_string_with_quotes() {
        let mut parser = JsonParser::new(r#""say \"hi\"""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("say \"hi\"".to_string()));
    }

    // Other tests carried over from week2 (adapted to work with JsonParser struct)

    #[test]
    fn test_parse_error_invalid_token() {
        let parser = JsonParser::new("@");
        assert!(parser.is_err());
    }

    #[test]
    fn test_parse_with_whitespace() -> Result<()> {
        let mut parser = JsonParser::new("  42  ")?;
        let result = parser.parse()?;
        assert_eq!(result, JsonValue::Number(42.0));

        let mut parser = JsonParser::new("\n\ttrue\n")?;
        let result = parser.parse()?;
        assert_eq!(result, JsonValue::Boolean(true));

        Ok(())
    }

    #[test]
    fn test_result_pattern_matching() -> Result<()> {
        let mut parser = JsonParser::new("42")?;
        let result = parser.parse();
        match result {
            Ok(JsonValue::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected successful number parse"),
        }

        let parser = JsonParser::new("@invalid@");
        match parser {
            Err(JsonError::UnexpectedToken { .. }) => {}
            _ => panic!("Expected UnexpectedTokenError error"),
        }

        Ok(())
    }
}
