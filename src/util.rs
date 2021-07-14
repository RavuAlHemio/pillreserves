use std::collections::HashMap;

use crate::model::Fraction;


pub(crate) fn gcd(mut a: i64, mut b: i64) -> i64 {
    if a < 0 {
        a = -a;
    }
    if b < 0 {
        b = -b;
    }

    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }

    a
}


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


pub(crate) struct FracToFloat;
impl tera::Filter for FracToFloat {
    fn filter(&self, value: &serde_json::Value, _args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
        let vstr = if let Some(vs) = value.as_str() {
            vs.to_owned()
        } else {
            value.to_string()
        };
        let frac: Fraction = vstr.parse()
            .map_err(|pfe| tera::Error::msg(pfe))?;
        Ok(serde_json::Value::from(frac.as_f64()))
    }
}
