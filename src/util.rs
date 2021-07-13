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
