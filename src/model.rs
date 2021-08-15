use derive_new::new;
use num_rational::Rational64;
use num_traits::Zero;
use serde::{Deserialize, Serialize};


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
    remaining: Rational64,
    dosage_morning: Rational64,
    dosage_noon: Rational64,
    dosage_evening: Rational64,
    dosage_night: Rational64,
    units_per_package: Rational64,
    packages_per_prescription: Rational64,
    show: bool,
    obverse_photo: Option<String>,
    reverse_photo: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, new, PartialEq, Serialize)]
pub(crate) struct DrugComponent {
    generic_name: String,
    amount: Rational64,
    unit: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, new, PartialEq, Serialize)]
pub(crate) struct DrugToDisplay {
    index: usize,
    drug: Drug,
    remaining_weeks: Option<i64>,
    weeks_per_prescription: Option<i64>,
}


impl Drug {
    pub fn trade_name(&self) -> &str { &self.trade_name }
    pub fn components(&self) -> &Vec<DrugComponent> { &self.components }
    pub fn description(&self) -> &str { &self.description }
    pub fn remaining(&self) -> Rational64 { self.remaining }
    pub fn dosage_morning(&self) -> Rational64 { self.dosage_morning }
    pub fn dosage_noon(&self) -> Rational64 { self.dosage_noon }
    pub fn dosage_evening(&self) -> Rational64 { self.dosage_evening }
    pub fn dosage_night(&self) -> Rational64 { self.dosage_night }
    pub fn units_per_package(&self) -> Rational64 { self.units_per_package }
    pub fn packages_per_prescription(&self) -> Rational64 { self.packages_per_prescription }
    pub fn show(&self) -> bool { self.show }
    pub fn obverse_photo(&self) -> Option<&str> { self.obverse_photo.as_ref().map(|s| s.as_str()) }
    pub fn reverse_photo(&self) -> Option<&str> { self.reverse_photo.as_ref().map(|s| s.as_str()) }

    pub fn total_dosage_day(&self) -> Rational64 {
        self.dosage_morning + self.dosage_noon + self.dosage_evening + self.dosage_night
    }

    pub fn units_per_prescription(&self) -> Rational64 {
        self.units_per_package * self.packages_per_prescription
    }

    pub fn reduce(&mut self, subtrahend: &Rational64) {
        let zero: Rational64 = Zero::zero();
        assert!(subtrahend > &zero);
        self.remaining = self.remaining - *subtrahend;
        if self.remaining < zero {
            self.remaining = zero;
        }
    }

    pub fn replenish(&mut self, addend: &Rational64) {
        let zero: Rational64 = Zero::zero();
        assert!(addend > &zero);
        self.remaining = self.remaining + *addend;
    }
}

impl DrugComponent {
    pub fn generic_name(&self) -> &str { &self.generic_name }
    pub fn amount(&self) -> Rational64 { self.amount }
    pub fn unit(&self) -> &str { &self.unit }
}

impl DrugToDisplay {
    pub fn index(&self) -> usize { self.index }
    pub fn drug(&self) -> &Drug { &self.drug }
    pub fn remaining_weeks(&self) -> Option<i64> { self.remaining_weeks }
    pub fn weeks_per_prescription(&self) -> Option<i64> { self.weeks_per_prescription }
}
