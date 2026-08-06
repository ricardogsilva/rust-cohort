use crate::error::JsonError;

// More about UTF-16 suplementary characters
// https://en.wikipedia.org/wiki/UTF-16
//
// first (AKA high, AKA leading) surrogate pair has a value in the range 0xD800 to 0xDBFF
const HIGH_SURROGATE_LOWER_BOUND: u32 = 0xD800;
const HIGH_SURROGATE_UPPER_BOUND: u32 = 0xDBFF;
// second (AKA low, AKA trailing) surrogate pair has a value in the range 0xDC00 to 0xDFFF
const LOW_SURROGATE_LOWER_BOUND: u32 = 0xDC00;
const LOW_SURROGATE_UPPER_BOUND: u32 = 0xDFFF;

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
            position: 0,
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
                '"' => tokens.push(self.tokenize_string()?),
                '0'..='9' | '-' => tokens.push(self.tokenize_number()?),
                't' | 'f' | 'n' => tokens.push(self.tokenize_keyword()?),
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

    fn tokenize_keyword(&mut self) -> Result<Token, JsonError> {
        // advances while trying to tokenize a JSON keyword
        // preallocate a size big enough to hold 'false'
        let mut keyword_as_string = String::with_capacity(5);
        while let Some(next_char) = self.peek() {
            match next_char {
                _ if next_char.is_alphabetic() => {
                    keyword_as_string.push(next_char);
                    self.advance();
                }
                _ => break, // next_char is no longer part of the keyword
            }
        }
        match keyword_as_string.as_str() {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            "null" => Ok(Token::Null),
            _ => Err(JsonError::UnexpectedToken {
                expected: "Either true, false or null".to_string(),
                found: keyword_as_string,
                position: self.position,
            }),
        }
    }

    fn tokenize_number(&mut self) -> Result<Token, JsonError> {
        // Advances while trying to tokenize a JSON number
        // preallocate a size that could fit foreseeable large numbers
        let mut number_as_string = String::with_capacity(24);
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
            Ok(value) => Ok(Token::Number(value)),
            Err(..) => Err(JsonError::InvalidNumber {
                value: number_as_string,
                position: self.position,
            }),
        }
    }

    fn tokenize_string(&mut self) -> Result<Token, JsonError> {
        self.advance(); // consume opening quote - throw it away
        // this capacity is really just a guess - seems likely that a most strings in 
        // a JSON document would be small
        let mut string_value = String::with_capacity(32);
        let mut string_terminated = false;
        while let Some(next_ch) = self.peek() {
            match next_ch {
                '\\' => {
                    string_value.push(self.parse_escape_sequence()?);
                }
                '"' => {
                    string_terminated = true;
                    self.advance();
                    break;
                }
                _ => {
                    string_value.push(next_ch);
                    self.advance();
                }
            }
        }
        match string_terminated {
            true => Ok(Token::String(string_value)),
            false => Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: self.position,
            }),
        }
    }

    fn parse_escape_sequence(&mut self) -> Result<char, JsonError> {
        self.advance(); // consume backslash
        match self.advance() {
            Some(current_ch) => match current_ch {
                '"' => Ok('\"'),
                '\\' => Ok('\\'),
                '/' => Ok('/'),
                'b' => Ok('\u{0008}'),
                'f' => Ok('\u{000C}'),
                'n' => Ok('\n'),
                'r' => Ok('\r'),
                't' => Ok('\t'),
                'u' => self.parse_unicode_escape_sequence(),
                _ => Err(JsonError::InvalidEscape {
                    char: current_ch,
                    position: self.position,
                }),
            },
            None => Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON escape character".to_string(),
                position: self.position,
            }),
        }
    }

    fn decode_utf16_surrogate_pair(high: u32, low: u32) -> Option<char> {
        // As per the wikipedia article linked at the top of this file,
        // decoding can be done like this (example: decode U+10437 (𐐷) from UTF-16):
        // Take high surrogate (0xD801) and subtract 0xD800, then multiply by 0x400, resulting in 0x0001 × 0x400 = 0x0400.
        // Take low surrogate (0xDC37) and subtract 0xDC00, resulting in 0x37.
        // Add these two results together (0x0437), and finally add 0x10000 to get the final code point, 0x10437.
        let pair_code_point = 0x10000
            + (high - HIGH_SURROGATE_LOWER_BOUND) * 0x400
            + (low - LOW_SURROGATE_LOWER_BOUND);
        char::from_u32(pair_code_point)
    }

    fn parse_unicode_escape_sequence(&mut self) -> Result<char, JsonError> {
        // already consumed the initial \u so next are the four hex digits of (the first) code point
        let first_hex = self.get_unicode_hex_string()?;
        let first_code_point = self.convert_hex_string_to_u32(&first_hex)?;
        if first_code_point >= HIGH_SURROGATE_LOWER_BOUND {
            if first_code_point > HIGH_SURROGATE_UPPER_BOUND {
                return Err(JsonError::InvalidUnicode {
                    sequence: first_hex,
                    position: self.position,
                });
            }
            let mut start_of_second_unicode_sequence = String::with_capacity(3);
            for _ in 0..2 {
                match self.advance() {
                    Some(ch) => {
                        start_of_second_unicode_sequence.push(ch);
                    }
                    None => {
                        return Err(JsonError::UnexpectedEndOfInput {
                            expected: "JSON value".to_string(),
                            position: self.position,
                        });
                    }
                }
            }
            if start_of_second_unicode_sequence != "\\u" {
                return Err(JsonError::InvalidUnicode {
                    sequence: start_of_second_unicode_sequence.to_string(),
                    position: self.position,
                });
            }
            let second_hex = self.get_unicode_hex_string()?;
            let second_code_point = self.convert_hex_string_to_u32(&second_hex)?;
            if !(LOW_SURROGATE_LOWER_BOUND..=LOW_SURROGATE_UPPER_BOUND).contains(&second_code_point)
            {
                return Err(JsonError::InvalidUnicode {
                    sequence: second_hex,
                    position: self.position,
                });
            }
            match Tokenizer::decode_utf16_surrogate_pair(first_code_point, second_code_point) {
                Some(v) => Ok(v),
                None => {
                    let error_sequence = (first_hex + &second_hex).to_string();
                    let error_position = self.position - error_sequence.len();
                    Err(JsonError::InvalidUnicode {
                        sequence: error_sequence,
                        position: error_position,
                    })
                }
            }
        } else {
            // its a normal code point, not part of a surrogate pair
            match char::from_u32(first_code_point) {
                Some(v) => Ok(v),
                None => Err(JsonError::InvalidUnicode {
                    sequence: first_hex,
                    position: self.position,
                }),
            }
        }
    }

    fn convert_hex_string_to_u32(&mut self, hex_string: &str) -> Result<u32, JsonError> {
        match u32::from_str_radix(hex_string, 16) {
            Ok(v) => Ok(v),
            Err(..) => Err(JsonError::InvalidUnicode {
                sequence: hex_string.to_string(),
                position: self.position,
            }),
        }
    }

    fn get_unicode_hex_string(&mut self) -> Result<String, JsonError> {
        let mut result = String::with_capacity(4);
        let mut char_count = 0;
        while char_count < 4 {
            match self.advance() {
                Some(ch) => result.push(ch),
                None => {
                    return Err(JsonError::InvalidUnicode {
                        sequence: result,
                        position: self.position,
                    });
                }
            }
            char_count += 1;
        }
        Ok(result)
    }

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
                n if n < self.input.len() => Some(self.input[self.position]),
                _ => None,
            },
        }
    }

    fn is_at_end(&self) -> bool {
        // check if we've consumed all input
        if self.input.is_empty() {
            true
        } else {
            self.position >= self.input.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::JsonError;
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
        assert_eq!(tokens, vec![Token::Number(42.0)]);
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
                    position, 3,
                    "error position should point to the start of 'xyz' (index 3), not past it"
                );
            }
            other => panic!("expected UnexpectedToken, got {:?}", other),
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
        let mut tokenizer = Tokenizer::new(r#""\u004""#);
        let tokens = tokenizer.tokenize();
        assert!(matches!(tokens, Err(JsonError::InvalidUnicode { .. })));
    }

    #[test]
    fn test_invalid_unicode_bad_hex() {
        let mut tokenizer = Tokenizer::new(r#""\u00GG""#);
        let tokens = tokenizer.tokenize();
        assert!(matches!(tokens, Err(JsonError::InvalidUnicode { .. })));
    }

    #[test]
    fn test_unterminated_string_with_escape() {
        let mut tokenizer = Tokenizer::new(r#""hello\n"#);
        let tokens = tokenizer.tokenize();
        assert!(tokens.is_err());
    }

    #[test]
    fn test_unicode_surrogate_pairs() {
        let mut tokenizer = Tokenizer::new(r#""\uD83D\uDE00""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("😀".to_string())]);
    }

    #[test]
    fn test_is_at_end() {
        let mut tokenizer = Tokenizer::new("1");
        assert_eq!(tokenizer.is_at_end(), false);
        tokenizer.advance();
        assert_eq!(tokenizer.is_at_end(), true);
    }

    #[test]
    fn test_advancing_sequence() {
        let mut tokenizer = Tokenizer::new("123");
        assert_eq!(tokenizer.advance(), Some('1'));
        assert_eq!(tokenizer.advance(), Some('2'));
        assert_eq!(tokenizer.advance(), Some('3'));
        assert_eq!(tokenizer.advance(), None);
    }

    #[test]
    fn test_peek_doesnt_advance() {
        let mut tokenizer = Tokenizer::new("ab");

        // Multiple peeks should return the same thing
        assert_eq!(tokenizer.peek(), Some('a'));
        assert_eq!(tokenizer.peek(), Some('a'));
        assert_eq!(tokenizer.peek(), Some('a'));

        // Position unchanged - advance still gets 'a'
        assert_eq!(tokenizer.advance(), Some('a'));
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
