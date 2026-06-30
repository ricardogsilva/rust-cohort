use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
    },
    UnexpectedEndOfInput {
        expected: String,
        position: usize,
    },
    InvalidNumber {
        value: String,
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

        assert!(!format!("{:?}", token_error).is_empty());
        assert!(!format!("{:?}", eof_error).is_empty());
        assert!(!format!("{:?}", num_error).is_empty());
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
}
