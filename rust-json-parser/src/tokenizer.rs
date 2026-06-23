use core::{f64, num};

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

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    // create an iterator that will provide each char in the input
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        // look at the next char in the iterator, without actually consuming it
        match ch {
            '{' => tokens.push(Token::LeftBrace),
            '}' => tokens.push(Token::RightBrace),
            ',' => tokens.push(Token::Comma),
            ':' => tokens.push(Token::Colon),
            '"' => {
                let mut string_value = String::new();
                chars.next();  // consume opening quote - throw it away

                while let Some(next_char) = chars.peek() {
                    match next_char {
                        '"' => break,  // end of string - don't consume closing quote
                        _ => {
                            string_value.push(*next_char);
                            chars.next();
                        },
                    }
                }
                tokens.push(Token::String(string_value));
            },
            '0'..='9' | '-' => {
                let mut number_as_string = String::new();

                number_as_string.push(ch);

                chars.next();

                // now look at the next chars to check whether they are also part of it or not
                while let Some(next_char) = chars.peek() {
                    match next_char {
                        '0'..='9' | '-' | '.' => {
                            number_as_string.push(*next_char);
                            chars.next();
                        },
                        _ => break,  // next_char is no longer part of a number

                    }
                }
                let number_value = number_as_string.parse::<f64>();
                match number_value {
                    Ok(value) => tokens.push(Token::Number(value)),
                    Err(err) => println!("Found an error while parsing {number_as_string} as a number: {err:?}"),
                }
                continue;  // already consumed the current char
            },
            't' | 'f' | 'n' => {
                let mut keyword_as_string = String::new();
                keyword_as_string.push(ch);
                chars.next();

                while let Some(next_char) = chars.peek() {
                    match next_char {
                        _ if next_char.is_alphabetic() => {
                            keyword_as_string.push(*next_char);
                            chars.next();
                        },
                        _ => break,  // next_char is not longer part of the keyword
                    }
                }
                match keyword_as_string.as_str() {
                    "true" => tokens.push(Token::Boolean(true)),
                    "false" => tokens.push(Token::Boolean(false)),
                    "null" => tokens.push(Token::Null),
                    _ => println!("Found an unexpected keyword {keyword_as_string}"),
                }
                continue;  // already consumed the current char
            },
            _ => {},
        }

        // after having looped through all the next chars, consume the current char
        chars.next();

    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_braces() {
        let tokens = tokenize("{}");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
    }

    #[test]
    fn test_simple_string() {
        let tokens = tokenize(r#""hello""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
    }

    #[test]
    fn test_tokenize_string() {
        let tokens = tokenize(r#""hello world""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
    }

    #[test]
    fn test_empty_string() {
        let tokens = tokenize(r#""""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
    }

    #[test]
    fn test_string_containing_json_special_characters() {
        let tokens = tokenize(r#""{key: value}""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
    }

    #[test]
    fn test_string_with_keyword_like_content() {
        let tokens = tokenize(r#""not true or false""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
    }

    #[test]
    fn test_string_with_number_like_content() {
        let tokens = tokenize(r#""phone: 555-1234""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
    }

    #[test]
    fn test_number() {
        let tokens = tokenize("42");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
    }

    #[test]
    fn test_negative_number() {
        let tokens = tokenize("-42");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
    }

    #[test]
    fn test_decimal_number() {
        let tokens = tokenize("0.5");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
    }

    #[test]
    fn test_leading_decimal_not_a_number() {
        let tokens = tokenize(".5");
        assert!(!tokens.contains(&Token::Number(0.5)));
    }

    #[test]
    fn test_boolean_and_null() {
        let tokens = tokenize("true false null");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
    }

    #[test]
    fn test_simple_object() {
        let tokens = tokenize(r#"{"name": "Alice"}"#);
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("name".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::String("Alice".to_string()));
        assert_eq!(tokens[4], Token::RightBrace);
    }

    #[test]
    fn test_multiple_values() {
        let tokens = tokenize(r#"{"age": 30, "active": true}"#);
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
    }
}