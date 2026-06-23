use core::f64;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    String(String),
    Number(f64),
    Boolean,
    Null,
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
            '"' => {
                chars.next();  // consume opening quote
                let mut string_value = String::new();
                // consume chars and put them in string_value until we find a closing quote
                while let Some(string_char) = chars.next() {
                    if string_char == '"' {
                        tokens.push(Token::String(string_value));
                        break
                    } else {
                        string_value.push(string_char);
                    }
                }

            }
            '0'..='9' | '-' => {
                let mut number_as_string = String::new();
                while let Some(number_char) = chars.next() {
                    match number_char {
                        '0'..='9' | '-' | '.' => number_as_string.push(number_char),
                        _ => break
                    }
                }
                let number_value = number_as_string.parse::<f64>();
                match number_value {
                    Ok(value) => tokens.push(Token::Number(value)),
                    Err(err) => println!("Found an error while parsing {number_as_string} as a number: {err:?}"),
                }
            }
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
}