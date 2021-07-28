use std::collections::HashMap;
use std::fmt;
use std::num::ParseIntError;

use num_rational::Rational64;


pub(crate) struct BrFilter;
impl tera::Filter for BrFilter {
    fn filter(&self, value: &serde_json::Value, _args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
        let vstr = if let Some(vs) = value.as_str() {
            vs.to_owned()
        } else {
            value.to_string()
        };
        Ok(serde_json::Value::String(vstr.replace("\n", "<br/>\n")))
    }
}


pub(crate) struct FracToStr;
impl tera::Filter for FracToStr {
    fn filter(&self, value: &serde_json::Value, _args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
        let value_array = match value.as_array() {
            Some(arr) => arr,
            None => return Err(tera::Error::msg("fraction value not an array")),
        };
        if value_array.len() != 2 {
            return Err(tera::Error::msg("fraction array has a length != 2"));
        }

        let numer = match value_array[0].as_i64() {
            Some(n) => n,
            None => return Err(tera::Error::msg("numerator (index 0) is not representable as an i64")),
        };
        let denom = match value_array[1].as_i64() {
            Some(n) => n,
            None => return Err(tera::Error::msg("denominator (index 1) is not representable as an i64")),
        };

        Ok(serde_json::Value::String(
            if denom == 1 {
                numer.to_string()
            } else {
                format!("{}/{}", numer, denom)
            }
        ))
    }
}


pub(crate) struct FracToFloat;
impl tera::Filter for FracToFloat {
    fn filter(&self, value: &serde_json::Value, _args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
        let value_array = match value.as_array() {
            Some(arr) => arr,
            None => return Err(tera::Error::msg("fraction value not an array")),
        };
        if value_array.len() != 2 {
            return Err(tera::Error::msg("fraction array has a length != 2"));
        }

        let numer = match value_array[0].as_i64() {
            Some(n) => n,
            None => return Err(tera::Error::msg("numerator (index 0) is not representable as an i64")),
        };
        let denom = match value_array[1].as_i64() {
            Some(n) => n,
            None => return Err(tera::Error::msg("denominator (index 1) is not representable as an i64")),
        };

        let numer_f64 = numer as f64;
        let denom_f64 = denom as f64;
        Ok(serde_json::Value::from(numer_f64 / denom_f64))
    }
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ParseDecimalError {
    TooManyDots(usize),
    MantissaParsing(ParseIntError),
    DenominatorTooLarge,
}
impl fmt::Display for ParseDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyDots(d)
                => write!(f, "too many ({}) dots", d),
            Self::MantissaParsing(e)
                => write!(f, "error parsing mantissa: {}", e),
            Self::DenominatorTooLarge
                => write!(f, "denominator too large"),
        }
    }
}
impl std::error::Error for ParseDecimalError {
}


pub(crate) fn parse_decimal(mut text: &str) -> Result<Rational64, ParseDecimalError> {
    let mut negate = false;
    if text.starts_with("-") {
        negate = true;
        text = &text[1..];
    }

    // count dots
    let dot_count = text.bytes()
        .filter(|b| *b == b'.')
        .count();
    if dot_count > 1 {
        return Err(ParseDecimalError::TooManyDots(dot_count));
    }

    // find position of dot
    let right_dot_pos = text.find('.')
        .unwrap_or(text.len());
    let power_of_ten = text.len() - right_dot_pos;

    // remove dot from text
    let text_no_dot = text.replace('.', "");

    // try parsing that as the mantissa
    let mut mantissa: i64 = text_no_dot.parse()
        .map_err(|e| ParseDecimalError::MantissaParsing(e))?;
    if negate {
        mantissa = -mantissa;
    }

    // get the denominator
    let mut denom: i64 = 1;
    for _ in 0..power_of_ten {
        denom = denom.checked_mul(10)
            .ok_or(ParseDecimalError::DenominatorTooLarge)?;
    }

    Ok(Rational64::new(mantissa, denom))
}
