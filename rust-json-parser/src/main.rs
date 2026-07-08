use rust_json_parser::{JsonError, parse_json};

fn main() {
    match parse_json(r#""The quick brown fox jumps over the lazy dog""#) {
        Ok(json_value) => println!("Valid str input: {:?}", json_value),
        Err(e) => println!("Got an unexpected error while parsing str: {e}"),
    };

    match parse_json("3.14159265358979") {
        Ok(json_value) => println!("Valid f64 input: {:?}", json_value),
        Err(e) => println!("Got an unexpected error while parsing f64 input: {e}"),
    }

    match parse_json(r#""missing end quote"#) {
        Ok(json_value) => panic!(
            "Parsing should have failed but succeeded with {:?}, something is wrong",
            json_value
        ),
        Err(e) => match e {
            JsonError::UnexpectedEndOfInput { .. } => {
                println!("Parsing has failed with the expected error of {:?}", e)
            }
            _ => panic!(
                "Parsing has failed as expected but did not produce the correct error. Got this instead {:?}",
                e
            ),
        },
    }
}
