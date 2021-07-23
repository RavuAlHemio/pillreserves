use std::collections::HashMap;


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
