use askama;
use num_rational::Rational64;


pub(crate) fn br<S: ToString>(s: S) -> askama::Result<String> {
    Ok(s.to_string().replace("\n", "<br/>\n"))
}

pub(crate) fn frac2str(frac: Rational64) -> askama::Result<String> {
    if *frac.denom() == 1 {
        Ok(frac.numer().to_string())
    } else {
        Ok(format!("{}/{}", frac.numer(), frac.denom()))
    }
}

pub(crate) fn frac2float(frac: Rational64) -> askama::Result<f64> {
    let numer_f64 = *frac.numer() as f64;
    let denom_f64 = *frac.denom() as f64;
    Ok(numer_f64 / denom_f64)
}
