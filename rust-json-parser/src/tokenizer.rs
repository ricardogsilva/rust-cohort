use crate::error::JsonError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Boolean(bool),
    Colon,
    Comma,
    LeftBrace,
    LeftBracket,
    Null,
    Number(f64),
    RightBrace,
    RightBracket,
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
}

impl Tokenizer {
    
    pub fn new(input: &str) -> Self {
        Tokenizer {
            // `collect` is a powerful iterator function
            // - It is able to exhaust an iterator and transform it to the annotated type
            input: input.chars().collect(),
            position: 0
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, JsonError> {
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(current) = self.peek() {
            match current {
                '{' => { 
                    tokens.push(Token::LeftBrace); 
                    self.advance();
                }
                '}' => {
                    tokens.push(Token::RightBrace);
                    self.advance();
                }
                '[' => {
                    tokens.push(Token::LeftBracket);
                    self.advance();
                }
                ']' => {
                    tokens.push(Token::RightBracket);
                    self.advance();
                }
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                }
                ':' => {
                    tokens.push(Token::Colon);
                    self.advance();
                }
                '"' => {
                    self.advance(); // consume opening quote - throw it away
                    let mut string_value = String::new();
                    let mut string_terminated = false;
                    let mut parsing_escape_sequence = false;
    
                    while let Some(next_ch) = self.peek() {
                        match parsing_escape_sequence {
                            true => {
                                match next_ch {
                                    '"' => string_value.push('\"'),
                                    '\\' => string_value.push('\\'),
                                    '/' => string_value.push('/'),
                                    'b' => string_value.push('\u{0008}'),
                                    'f' => string_value.push('\u{000C}'),
                                    'n' => string_value.push('\n'),
                                    'r' => string_value.push('\r'),
                                    't' => string_value.push('\t'),
                                    'u' => {  // may be unicode
                                        self.advance();  // throw away the 'u'
                                        let mut unicode_hex_string = String::new();
                                        let mut unicode_char_count = 0;
                                        while unicode_char_count < 4 {
                                            match self.advance() {
                                                Some(ch) => unicode_hex_string.push(ch),
                                                None => return Err(
                                                    JsonError::InvalidUnicode { 
                                                        sequence: unicode_hex_string.clone(), 
                                                        position: self.position 
                                                    }
                                                )
                                            }
                                            unicode_char_count += 1;
                                        }
                                        match u32::from_str_radix(&unicode_hex_string, 16) {
                                            Ok(value) => {
                                                match char::from_u32(value) {
                                                    Some(v) => string_value.push(v),
                                                    None => return Err(
                                                        JsonError::InvalidUnicode { 
                                                            sequence: unicode_hex_string, 
                                                            position: self.position 
                                                        }
                                                    )
                                                }
                                                parsing_escape_sequence = false;
                                                continue;
                                            }
                                            Err(..) => return Err(
                                                JsonError::InvalidUnicode { 
                                                    sequence: unicode_hex_string, 
                                                    position: self.position 
                                                }
                                            ),
                                        }
                                    }
                                    _ => return Err(JsonError::InvalidEscape { char: next_ch, position: self.position })
                                }
                                parsing_escape_sequence = false;
                                self.advance();
                            }
                            false => {
                                match next_ch {
                                    '\\' => {
                                        parsing_escape_sequence = true;
                                        self.advance();
                                        continue;
                                    }
                                    '"' => {
                                        self.advance();
                                        string_terminated = true;
                                        break;
                                    }
                                    _ => {
                                        string_value.push(next_ch);
                                        self.advance();
                                    }
                                }
                            }
                        }
                    }
                    if !string_terminated {
                        return Err(JsonError::UnexpectedEndOfInput {
                            expected: "JSON value".to_string(),
                            position: self.position,
                        });
                    }
                    tokens.push(Token::String(string_value));
                }
                '0'..='9' | '-' => {
                    let mut number_as_string = String::new();
                    number_as_string.push(current);
                    self.advance();
    
                    // now look at the next chars to check whether they are also part of it or not
                    while let Some(next_char) = self.peek() {
                        match next_char {
                            '0'..='9' | '-' | '.' => {
                                number_as_string.push(next_char);
                                self.advance();
                            }
                            _ => break, // next_char is no longer part of number
                        }
                    }
                    let number_value = number_as_string.parse::<f64>();
                    match number_value {
                        Ok(value) => tokens.push(Token::Number(value)),
                        Err(..) => {
                            return Err(JsonError::InvalidNumber {
                                value: number_as_string,
                                position: self.position,
                            });
                        }
                    }
                }
                't' | 'f' | 'n' => {
                    let mut keyword_as_string = String::new();
                    keyword_as_string.push(current);
                    self.advance();
    
                    while let Some(next_char) = self.peek() {
                        match next_char {
                            _ if next_char.is_alphabetic() => {
                                keyword_as_string.push(next_char);
                                self.advance();
                            }
                            _ => break, // next_char is not longer part of the keyword
                        }
                    }
                    match keyword_as_string.as_str() {
                        "true" => tokens.push(Token::Boolean(true)),
                        "false" => tokens.push(Token::Boolean(false)),
                        "null" => tokens.push(Token::Null),
                        _ => {
                            return Err(JsonError::UnexpectedToken {
                                expected: "Either true, false or null".to_string(),
                                found: keyword_as_string,
                                position: self.position,
                            });
                        }
                    }
                }
                ' ' | '\n' | '\r' | '\t' => {
                    self.advance();
                } // whitespace does not need to be captured
                _ => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "valid JSON token".to_string(),
                        found: current.to_string(),
                        position: self.position,
                    });
                }
            }
        }
        Ok(tokens)
    }

    // private helpers

    fn advance(&mut self) -> Option<char> {
        // move forward, return previous char
        match self.is_at_end() {
            true => None,
            false => {
                self.position += 1;
                Some(self.input[self.position - 1])
            }
        }
    }
    
    fn peek(&self) -> Option<char> {
        // look at current char without advancing
        match self.is_at_end() {
            true => None,
            false => match self.position {
                n if n <= self.input.len() - 1 => Some(self.input[self.position]),
                _ => None
            }
        }
    }
    
    fn is_at_end(&self) -> bool {
        // check if we've consumed all input
        if self.input.len() == 0 {
            true
        } else {
            self.position >= self.input.len()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::JsonError};
    use std::assert_matches;

    type Result<T> = std::result::Result<T, JsonError>;

    #[test]
    fn test_tokenizer_creation() {
        let _tokenizer = Tokenizer::new(r#"hello"#);
    }

    #[test]
    fn test_tokenize_number() {
        let mut tokenizer = Tokenizer::new("42");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(
            tokens, 
            vec![Token::Number(42.0)]
        );
    }

    #[test]
    fn test_tokenize_literals() {
        let mut t1 = Tokenizer::new("true");
        assert_eq!(t1.tokenize().unwrap(), vec![Token::Boolean(true)]);
        
        let mut t2 = Tokenizer::new("false");
        assert_eq!(t2.tokenize().unwrap(), vec![Token::Boolean(false)]);
        
        let mut t3 = Tokenizer::new("null");
        assert_eq!(t3.tokenize().unwrap(), vec![Token::Null]);
    }

    #[test]
    fn test_tokenize_simple_string() {
        let mut tokenizer = Tokenizer::new(r#""hello""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("hello".to_string())]);
    }

    #[test]
    fn test_tokenizer_multiple_tokens() {
        // test that a single tokenize() call  handles multiple tokens
        let mut tokenizer = Tokenizer::new("123 456");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
    }
    
    #[test]
    fn test_tokenize_negative_number() {
        let mut tokenizer = Tokenizer::new("-3.14");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Number(-3.14)]);
    }
    
    #[test]
    fn test_invalid_keyword_error_position_points_to_start() {
        let input = r#"   xyz"#;
        let mut tokenizer = Tokenizer::new(input);
        let err = tokenizer.tokenize().unwrap_err();
        match err {
            JsonError::UnexpectedToken { position, .. } => {
                assert_eq!(
                    position, 
                    3, 
                    "error position should point to the start of 'xyz' (index 3), not past it"
                );
            }
            other => panic!("expected UnexpectedToken, got {:?}", other)
        }
    }

    #[test]
    fn test_escape_newline() {
        let mut tokenizer = Tokenizer::new(r#""hello\nworld""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("hello\nworld".to_string())]);
    }
    
    #[test]
    fn test_escape_tab() {
        let mut tokenizer = Tokenizer::new(r#""col1\tcol2""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("col1\tcol2".to_string())]);
    }
    
    #[test]
    fn test_escape_quote() {
        let mut tokenizer = Tokenizer::new(r#""say \"hello\"""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("say \"hello\"".to_string())]);
    }
    
    #[test]
    fn test_escape_backslash() {
        let mut tokenizer = Tokenizer::new(r#""path\\to\\file""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("path\\to\\file".to_string())]);
    }
    
    #[test]
    fn test_multiple_escapes() {
        let mut tokenizer = Tokenizer::new(r#""a\nb\tc\"""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("a\nb\tc\"".to_string())]);
    }

    #[test]
    fn test_escape_forward_slash() {
        let mut tokenizer = Tokenizer::new(r#""a\/b""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("a/b".to_string())]);
    }
    
    #[test]
    fn test_escape_carriage_return() {
        let mut tokenizer = Tokenizer::new(r#""line\r\n""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("line\r\n".to_string())]);
    }
    
    #[test]
    fn test_escape_backspace_formfeed() {
        let mut tokenizer = Tokenizer::new(r#""\b\f""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("\u{0008}\u{000C}".to_string())]);
    }

    #[test]
    fn test_unicode_escape_basic() {
        // '\u0041' is 'A'
        let mut tokenizer = Tokenizer::new(r#""\u0041""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("A".to_string())]);
    }
    
    #[test]
    fn test_unicode_escape_multiple() {
        // '\u0048\u0069' is 'Hi'
        let mut tokenizer = Tokenizer::new(r#""\u0048\u0069""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("Hi".to_string())]);
    }
    
    #[test]
    fn test_unicode_escape_mixed() {
        // '\u0057' is 'W'
        let mut tokenizer = Tokenizer::new(r#""Hello \u0057orld""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("Hello World".to_string())]);
    }
    
    #[test]
    fn test_unicode_escape_lowercase() {
        // if hex digit is in lowercase it should also work
        // '\u004a' is 'J' 
        let mut tokenizer = Tokenizer::new(r#""\u004a""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("J".to_string())]);
    }

    #[test]
    fn test_invalid_escape_sequence() {
        let mut tokenizer = Tokenizer::new(r#""\q""#);
        let tokens = tokenizer.tokenize();
        assert!(matches!(tokens, Err(JsonError::InvalidEscape { .. })));
    }
    
    #[test]
    fn test_invalid_unicode_too_short() {
        let mut tokenizer = Tokenizer::new(r#""\u{004}""#);
        let tokens = tokenizer.tokenize();
        assert!(matches!(tokens, Err(JsonError::InvalidUnicode { .. })));
    }
    
    #[test]
    fn test_invalid_unicode_bad_hex() {
        let mut tokenizer = Tokenizer::new(r#""\u{00GG}""#);
        let tokens = tokenizer.tokenize();
        assert!(matches!(tokens, Err(JsonError::InvalidUnicode { .. })));
    }
    
    #[test]
    fn test_unterminated_string_with_escape() {
        let mut tokenizer = Tokenizer::new(r#""hello\n"#);
        let tokens = tokenizer.tokenize();
        assert!(tokens.is_err());
    }

    // tests carried over from week2 (adapted to work with Tokenizer struct)

    // string boundary tests

    #[test]
    fn test_empty_string() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_containing_json_special_chars() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""{key: value}""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_keyword_like_content() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""not true or false""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_number_like_content() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""phone: 555-1234""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
        Ok(())
    }

    // number parsing tests

    #[test]
    fn test_negative_number() -> Result<()> {
        let mut tokenizer = Tokenizer::new("-42");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
        Ok(())
    }

    #[test]
    fn test_decimal_number() -> Result<()> {
        let mut tokenizer = Tokenizer::new("0.5");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
        Ok(())
    }

    // test parsing errors

    #[test]
    fn test_leading_decimal_not_a_number() {
        // .5 is invalid JSON
        let mut tokenizer = Tokenizer::new(".5");
        let err = tokenizer.tokenize().unwrap_err();
        assert!(matches!(
            err,
            JsonError::UnexpectedToken { position: 0, .. }
        ));
    }

    #[test]
    fn test_unterminated_string() {
        let mut tokenizer = Tokenizer::new(r#""missing end quote"#);
        let err = tokenizer.tokenize().unwrap_err();
        assert_matches!(
            err,
            JsonError::UnexpectedEndOfInput { position: 18, .. },
            "Expected UnexpectedEndOfInput error, got {}",
            err
        );
    }

    // other tests from week1

    #[test]
    fn test_empty_braces() -> Result<()> {
        let mut tokenizer = Tokenizer::new("{}");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
        Ok(())
    }

    #[test]
    fn test_simple_string() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""hello""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        Ok(())
    }

    #[test]
    fn test_tokenize_string() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""hello world""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
        Ok(())
    }

    #[test]
    fn test_number() -> Result<()> {
        let mut tokenizer = Tokenizer::new("42");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
        Ok(())
    }

    #[test]
    fn test_boolean_and_null() -> Result<()> {
        let mut tokenizer = Tokenizer::new("true false null");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
        Ok(())
    }

    #[test]
    fn test_simple_object() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#"{"name": "Alice"}"#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("name".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::String("Alice".to_string()));
        assert_eq!(tokens[4], Token::RightBrace);
        Ok(())
    }

    #[test]
    fn test_multiple_values() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#"{"age": 30, "active": true}"#);
        let tokens = tokenizer.tokenize()?;
        println!("{tokens:?}");
        assert_eq!(tokens.len(), 9);
        // note: Instead of testing containment, since we have a small input,
        // this verifies all tokens positionally
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("age".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::Number(30.0));
        assert_eq!(tokens[4], Token::Comma);
        assert_eq!(tokens[5], Token::String("active".to_string()));
        assert_eq!(tokens[6], Token::Colon);
        assert_eq!(tokens[7], Token::Boolean(true));
        assert_eq!(tokens[8], Token::RightBrace);
        Ok(())
    }

    #[test]
    fn test_array() -> Result<()> {
        let mut tokenizer = Tokenizer::new("[1, 2, 3]");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], Token::LeftBracket);
        assert_eq!(tokens[1], Token::Number(1.0));
        assert_eq!(tokens[2], Token::Comma);
        assert_eq!(tokens[3], Token::Number(2.0));
        assert_eq!(tokens[4], Token::Comma);
        assert_eq!(tokens[5], Token::Number(3.0));
        assert_eq!(tokens[6], Token::RightBracket);
        Ok(())
    }

    #[test]
    fn test_nested_object() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#"{"nested": {"name": "Alice"}, "age": 30}"#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 13);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("nested".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::LeftBrace);
        assert_eq!(tokens[4], Token::String("name".to_string()));
        assert_eq!(tokens[5], Token::Colon);
        assert_eq!(tokens[6], Token::String("Alice".to_string()));
        assert_eq!(tokens[7], Token::RightBrace);
        assert_eq!(tokens[8], Token::Comma);
        assert_eq!(tokens[9], Token::String("age".to_string()));
        assert_eq!(tokens[10], Token::Colon);
        assert_eq!(tokens[11], Token::Number(30.0));
        assert_eq!(tokens[12], Token::RightBrace);
        Ok(())
    }

    #[test]
    fn test_number_zero() -> Result<()> {
        let mut tokenizer = Tokenizer::new("0");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.0));
        Ok(())
    }

    #[test]
    fn test_complex_input() -> Result<()> {
        let input = r#"
        {
          "name": "Alice Johnson",
          "age": 28,
          "email": "alice@example.com",
          "active": true,
          "preferences": {
            "theme": "dark",
            "notifications": true,
            "language": "en"
          },
          "tags": [
            "developer", 
            "rust", 
            "python"
          ],
          "metadata": {
            "created": "2023-01-15T10:30:00Z",
            "updated": "2023-12-01T15:45:30Z"
          }
        }
        "#;
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize()?;
        println!("{tokens:?}");
        assert_eq!(tokens.len(), 55);
        assert_eq!(tokens[1], Token::String("name".to_string()));
        assert_eq!(tokens[5], Token::String("age".to_string()));
        assert_eq!(tokens[10], Token::Colon);
        assert_eq!(tokens[15], Token::Boolean(true));
        assert_eq!(tokens[21], Token::Colon);
        assert_eq!(tokens[26], Token::Boolean(true));
        assert_eq!(tokens[30], Token::String("en".to_string()));
        assert_eq!(tokens[36], Token::String("developer".to_string()));
        assert_eq!(tokens[42], Token::Comma);
        assert_eq!(tokens[46], Token::String("created".to_string()));
        assert_eq!(tokens[50], Token::String("updated".to_string()));
        assert_eq!(tokens[54], Token::RightBrace);
        Ok(())
    }
}
