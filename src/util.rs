use std::fmt;
use std::num::ParseIntError;

use num_rational::Rational64;


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
    let dot_count = text.chars()
        .filter(|c| *c == '.')
        .count();
    if dot_count > 1 {
        return Err(ParseDecimalError::TooManyDots(dot_count));
    }

    // find position of dot
    let power_of_ten = if let Some(right_dot_pos) = text.find('.') {
        text.len() - (right_dot_pos + 1)
    } else {
        0
    };

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


#[cfg(test)]
mod tests {
    fn test_parse_decimal(expnum: i64, expden: i64, text: &str) {
        let rat = super::parse_decimal(text)
            .unwrap();
        assert_eq!(expnum, *rat.numer());
        assert_eq!(expden, *rat.denom());
    }

    #[test]
    fn test_parse_no_dot() {
        test_parse_decimal(0, 1, "0");
        test_parse_decimal(5, 1, "5");
        test_parse_decimal(128, 1, "128");
        test_parse_decimal(-5, 1, "-5");
        test_parse_decimal(-128, 1, "-128");
    }

    #[test]
    fn test_parse_trailing_dot() {
        test_parse_decimal(0, 1, "0.");
        test_parse_decimal(5, 1, "5.");
        test_parse_decimal(128, 1, "128.");
        test_parse_decimal(-5, 1, "-5.");
        test_parse_decimal(-128, 1, "-128.");
    }

    #[test]
    fn test_parse_leading_dot() {
        test_parse_decimal(0, 1, ".0");
        test_parse_decimal(1, 2, ".5");
        test_parse_decimal(16, 125, ".128");
        test_parse_decimal(-1, 2, "-.5");
        test_parse_decimal(-16, 125, "-.128");
    }

    #[test]
    fn test_parse_leading_zero_dot() {
        test_parse_decimal(0, 1, "0.0");
        test_parse_decimal(1, 2, "0.5");
        test_parse_decimal(16, 125, "0.128");
        test_parse_decimal(-1, 2, "-0.5");
        test_parse_decimal(-16, 125, "-0.128");
    }

    #[test]
    fn test_parse_mid_dot() {
        test_parse_decimal(32, 25, "1.28");
        test_parse_decimal(64, 5, "12.8");
        test_parse_decimal(-32, 25, "-1.28");
        test_parse_decimal(-64, 5, "-12.8");
    }
}
