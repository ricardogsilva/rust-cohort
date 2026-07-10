use rust_json_parser::{JsonError, JsonParser};

fn main() {
    // note how `Ok(mut parser)` is valid rust.
    // we could also write like this:
    //
    //
    // let parser = match JsonParser::new(r#""The quick brown fox jumps over the lazy dog""#) {
    //     Ok(mut p) => {
    //         match p.parse {}
    //     }
    // }
    //
    // // now use `parser` to do immutable things
    //
    // In this example code above, `p` is mutable but `parser` is not
    match JsonParser::new(r#""The quick brown fox jumps over the lazy dog""#) {
        Ok(mut parser) => match parser.parse() {
            Ok(value) => println!("Valid str input: {:?}", value),
            Err(e) => eprintln!("Got an unexpected error while parsing input: {e}"),
        },
        Err(e) => eprintln!("Got an unexpected error while tokenizing input: {e}"),
    };

    match JsonParser::new("3.14159265358979") {
        Ok(mut parser) => match parser.parse() {
            Ok(value) => println!("Valid f64 input: {:?}", value),
            Err(e) => eprintln!("Got an unexpected error while parsing input: {e}"),
        },
        Err(e) => eprintln!("Got an unexpected error while tokenizing input: {e}"),
    }

    match JsonParser::new(r#""missing end quote"#) {
        Ok(mut parser) => match parser.parse() {
            Ok(value) => panic!(
                "Tokenizing and parsing should have failed but succeeded with {:?}, something is wrong",
                value
            ),
            Err(e) => panic!(
                "Tokenizing should have failed but succeeded with {:?} and then parsing failed with {:?}, something is wrong",
                parser, e
            ),
        },
        Err(e) => match e {
            JsonError::UnexpectedEndOfInput { .. } => {
                println!("Tokenizing has failed with the expected error of {:?}", e)
            }
            _ => panic!(
                "Tokenizing has failed as expected but did not produce the correct error. Got this instead {:?}",
                e
            ),
        },
    };
}
