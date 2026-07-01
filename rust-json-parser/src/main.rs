use rust_json_parser::{Result, parse_json};

fn main() -> Result<()> {
    let valid_str_input = r#""The quick brown fox jumps over the lazy dog""#;
    let valid_f64_input = "3.14159265358979";
    let invalid_str_input = r#""missing end quote"#;

    let valid_str_result = parse_json(valid_str_input)?;
    println!("Valid str input: {valid_str_input}");
    println!("result: {:?}", valid_str_result);

    let valid_f64_result = parse_json(valid_f64_input)?;
    println!("Valid f64 input: {valid_f64_input}");
    println!("result: {:?}", valid_f64_result);

    let invalid_str_result = parse_json(invalid_str_input).unwrap_err();
    println!("Invalid str input: {invalid_str_input}");
    println!("result: {:?}", invalid_str_result);

    Ok(())
}
