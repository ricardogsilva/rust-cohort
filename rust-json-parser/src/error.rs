use std::fmt;

/// Used for providing informative errors on both the tokenizing and parsing of input JSON
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    /// A token was found where a different one was expected — e.g. a stray
    /// symbol, a misplaced comma, or a value where a key was required.
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
    },
    /// The input ended while a token, value, or closing bracket/quote was
    /// still expected — e.g. a truncated string, object, or array.
    UnexpectedEndOfInput {
        expected: String,
        position: usize,
    },
    /// A numeric literal was malformed, such as extra decimal points or
    /// stray characters, and could not be parsed into an `f64`.
    InvalidNumber {
        value: String,
        position: usize,
    },
    /// A `\<char>` escape sequence inside a string used a character that
    /// isn't a recognized JSON escape (e.g. `\q`).
    InvalidEscape {
        char: char,
        position: usize,
    },
    /// A `\u` escape inside a string wasn't followed by four valid hex
    /// digits, or formed an invalid/incomplete surrogate pair.
    InvalidUnicode {
        sequence: String,
        position: usize,
    },
}

// Display trait for JsonError
// Why do we need this? Because it enables using the `.to_string()` method and
// also enables calling `format!("{}")`
impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Expected: {} - found: {} - position {}",
                    expected, found, position
                )
            }
            JsonError::UnexpectedEndOfInput { expected, position } => {
                write!(f, "Expected: {} - position {}", expected, position)
            }
            JsonError::InvalidNumber { value, position } => {
                write!(f, "Value: {} - position {}", value, position)
            }
            JsonError::InvalidEscape { char, position } => {
                write!(f, "Invalid escape char: {} - position {}", char, position)
            }
            JsonError::InvalidUnicode { sequence, position } => {
                write!(
                    f,
                    "Invalid Unicode sequence: {} - position {}",
                    sequence, position
                )
            }
        }
    }
}

// Error trait for JsonError
// Why do we need this? Because all errors must implement the Error trait in
// order to be usable in a Result<T, E>.
// The implementation is just this oneliner because the `Error` trait only requires
// that both the `Debug` and the `Display` traits be implemented - We already
// have those two in this case
impl std::error::Error for JsonError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = JsonError::UnexpectedToken {
            expected: "number".to_string(),
            found: "@".to_string(),
            position: 5,
        };
        // the "{:?}" syntax is how we do a debug print of something
        // in order to work, the corresponding type needs to have an
        // implementation of the `Debug` trait
        assert!(format!("{:?}", error).contains("UnexpectedToken"));
    }

    #[test]
    fn test_error_variants() {
        let token_error = JsonError::UnexpectedToken {
            expected: "number".to_string(),
            found: "x".to_string(),
            position: 3,
        };

        let eof_error = JsonError::UnexpectedEndOfInput {
            expected: "closing quote".to_string(),
            position: 10,
        };

        let num_error = JsonError::InvalidNumber {
            value: "12.34.56".to_string(),
            position: 0,
        };

        let escape_error = JsonError::InvalidEscape {
            char: 'b',
            position: 0,
        };
        let unicode_error = JsonError::InvalidUnicode {
            sequence: "something".to_string(),
            position: 0,
        };

        assert!(!format!("{:?}", token_error).is_empty());
        assert!(!format!("{:?}", eof_error).is_empty());
        assert!(!format!("{:?}", num_error).is_empty());
        assert!(!format!("{:?}", escape_error).is_empty());
        assert!(!format!("{:?}", unicode_error).is_empty());
    }

    #[test]
    fn test_error_display() {
        let error = JsonError::UnexpectedToken {
            expected: "valid JSON".to_string(),
            found: "@".to_string(),
            position: 0,
        };

        let message = format!("{}", error);
        assert!(message.contains("position 0"));
        assert!(message.contains("valid JSON"));
        assert!(message.contains("@"));
    }

    #[test]
    fn test_invalid_escape_display() {
        let err = JsonError::InvalidEscape {
            char: 'q',
            position: 5,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("escape"));
        assert!(msg.contains("q"));
    }

    #[test]
    fn test_invalid_unicode_display() {
        let err = JsonError::InvalidUnicode {
            sequence: "00GG".to_string(),
            position: 3,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("unicode") || msg.contains("Unicode"));
    }

    #[test]
    fn test_error_is_std_error() {
        let err = JsonError::InvalidEscape {
            char: 'x',
            position: 0,
        };
        // this is checking if the error implements the Error trait somehow
        let _: &dyn std::error::Error = &err;
    }
}
