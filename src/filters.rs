use std::collections::HashMap;

use serde_json::value::{to_value, Value};
use tera::{Error, Result};

/// Renders float with zero in decimal part
pub fn with_zero(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let placeholder = 0.0;
    let new_value = match value {
        Value::Number(number) => match number.as_f64() {
            Some(float) => {
                if float.fract() == placeholder {
                    let result = format!("{:.1}", float);
                    result
                } else {
                    let result = format!("{}", float);
                    result
                }
            }
            None => {
                let result = format!("{}", placeholder);
                result
            }
        },
        _ => return Err(Error::msg("Filter `with_zero` received non-float type")),
    };

    Ok(to_value(new_value).unwrap())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::value::to_value;

    use super::*;

    #[test]
    fn test_with_zero() {
        let tests: Vec<(f64, String)> = vec![
            (300.0, String::from("300.0")),
            (-400.03, String::from("-400.03")),
        ];
        for (input, expected) in tests {
            let result = with_zero(&to_value(input).unwrap(), &HashMap::new());

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), to_value(expected).unwrap());
        }

        // integers are treated as floats
        let result = with_zero(&to_value(600u16).unwrap(), &HashMap::new());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value("600.0").unwrap());

        // unsupported type
        let result = with_zero(&to_value("500.0").unwrap(), &HashMap::new());

        assert!(result.is_err());
    }
}
