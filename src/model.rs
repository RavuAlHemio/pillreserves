use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;
use std::ops;

use derive_new::new;
use serde::{Deserialize, Serialize};
use serde::de::Visitor;

use crate::util::gcd;


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Fraction {
    num: i64,
    den: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, new, PartialEq, Serialize)]
pub(crate) struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub data_path: String,
    pub auth_tokens: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, new, PartialEq, Serialize)]
pub(crate) struct Drug {
    trade_name: String,
    components: Vec<DrugComponent>,
    description: String,
    remaining: Fraction,
    dosage_morning: Fraction,
    dosage_noon: Fraction,
    dosage_evening: Fraction,
    dosage_night: Fraction,
    show: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, new, PartialEq, Serialize)]
pub(crate) struct DrugComponent {
    generic_name: String,
    amount: Fraction,
    unit: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, new, PartialEq, Serialize)]
pub(crate) struct DrugToDisplay {
    index: usize,
    drug: Drug,
    remaining_weeks: Option<i64>,
}


impl Fraction {
    pub fn new(
        mut num: i64,
        mut den: i64,
    ) -> Fraction {
        if den < 0 {
            num *= -1;
            den *= -1;
        }
        if den == 0 {
            panic!("division by zero");
        }

        let gcd = gcd(num, den);
        Fraction {
            num: num / gcd,
            den: den / gcd,
        }
    }

    pub fn num(&self) -> i64 { self.num }
    pub fn den(&self) -> i64 { self.den }
}

impl Drug {
    pub fn trade_name(&self) -> &str { &self.trade_name }
    pub fn components(&self) -> &Vec<DrugComponent> { &self.components }
    pub fn description(&self) -> &str { &self.description }
    pub fn remaining(&self) -> Fraction { self.remaining }
    pub fn dosage_morning(&self) -> Fraction { self.dosage_morning }
    pub fn dosage_noon(&self) -> Fraction { self.dosage_noon }
    pub fn dosage_evening(&self) -> Fraction { self.dosage_evening }
    pub fn dosage_night(&self) -> Fraction { self.dosage_night }
    pub fn show(&self) -> bool { self.show }

    pub fn total_dosage_day(&self) -> Fraction {
        self.dosage_morning + self.dosage_noon + self.dosage_evening + self.dosage_night
    }

    pub fn reduce(&mut self, subtrahend: &Fraction) {
        self.remaining = self.remaining - *subtrahend;
        let zero = Fraction::new(0, 1);
        if self.remaining < zero {
            self.remaining = zero;
        }
    }

    pub fn replenish(&mut self, addend: &Fraction) {
        self.remaining = self.remaining + *addend;
    }
}

impl DrugComponent {
    pub fn generic_name(&self) -> &str { &self.generic_name }
    pub fn amount(&self) -> Fraction { self.amount }
    pub fn unit(&self) -> &str { &self.unit }
}

impl DrugToDisplay {
    pub fn index(&self) -> usize { self.index }
    pub fn drug(&self) -> &Drug { &self.drug }
    pub fn remaining_weeks(&self) -> Option<i64> { self.remaining_weeks }
}

struct FractionVisitor;
impl<'de> Visitor<'de> for FractionVisitor {
    type Value = Fraction;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string with a numerator and a denominator as decimal numbers separated with a slash")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        let slash_pieces: Vec<&str> = v.split('/').collect();
        if slash_pieces.len() == 1 {
            let num: i64 = slash_pieces[0].parse()
                .map_err(|e| E::custom(e))?;
            Ok(Fraction::new(num, 1))
        } else if slash_pieces.len() == 2 {
            let num: i64 = slash_pieces[0].parse()
                .map_err(|e| E::custom(e))?;
            let den: i64 = slash_pieces[1].parse()
                .map_err(|e| E::custom(e))?;
            Ok(Fraction::new(num, den))
        } else {
            Err(E::custom("too many slashes in value"))
        }
    }
}

impl<'de> Deserialize<'de> for Fraction {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Fraction, D::Error>
    {
        deserializer.deserialize_str(FractionVisitor)
    }
}
impl Serialize for Fraction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
    {
        if self.den == 1 {
            serializer.serialize_str(&format!("{}", self.num))
        } else {
            serializer.serialize_str(&format!("{}/{}", self.num, self.den))
        }
    }
}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.num * other.den).cmp(&(self.den * other.num)))
    }
}
impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl ops::Neg for Fraction {
    type Output = Fraction;
    fn neg(self) -> Self::Output {
        Fraction::new(-self.num, self.den)
    }
}
impl ops::Add for Fraction {
    type Output = Fraction;
    fn add(self, rhs: Self) -> Self::Output {
        let new_num = self.num * rhs.den + rhs.num * self.den;
        let new_den = self.den * rhs.den;
        Fraction::new(new_num, new_den)
    }
}
impl ops::Sub for Fraction {
    type Output = Fraction;
    fn sub(self, rhs: Self) -> Self::Output {
        let new_num = self.num * rhs.den - rhs.num * self.den;
        let new_den = self.den * rhs.den;
        Fraction::new(new_num, new_den)
    }
}
impl ops::Mul for Fraction {
    type Output = Fraction;
    fn mul(self, rhs: Self) -> Self::Output {
        let new_num = self.num * rhs.num;
        let new_den = self.den * rhs.den;
        Fraction::new(new_num, new_den)
    }
}
impl ops::Div for Fraction {
    type Output = Fraction;
    fn div(self, rhs: Self) -> Self::Output {
        let new_num = self.num * rhs.den;
        let new_den = self.den * rhs.num;
        Fraction::new(new_num, new_den)
    }
}
