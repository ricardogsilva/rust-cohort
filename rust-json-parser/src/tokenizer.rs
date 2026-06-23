#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    String,
    Number,
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
}