use std::collections::HashMap;

use derive_new::new;
use num_rational::Rational64;
use num_traits::Zero;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Deserialize, Eq, new, PartialEq, Serialize)]
pub(crate) struct Config {
    pub listen_addr: String,
    pub base_url: String,
    pub data_path: String,
    pub auth_tokens: Vec<String>,
    pub column_profiles: HashMap<String, Vec<String>>,
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
    #[serde(default)] is_pill: bool,
    #[serde(default = "Drug::default_in_replenishment_cycle")] in_replenishment_cycle: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, new, PartialEq, Serialize)]
pub(crate) struct DrugComponent {
    generic_name: String,
    amount: Rational64,
    unit: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, new, PartialEq, Serialize)]
pub(crate) struct DrugToDisplay {
    pub index: usize,
    pub drug: Drug,
    pub remaining_weeks: Option<i64>,
    pub weeks_per_prescription: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, new, PartialEq, Serialize)]
pub(crate) struct DailyPills {
    morning: u64,
    noon: u64,
    evening: u64,
    night: u64,
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
    pub fn is_pill(&self) -> bool { self.is_pill }
    pub fn in_replenishment_cycle(&self) -> bool { self.in_replenishment_cycle }

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

    pub fn default_in_replenishment_cycle() -> bool { true }
}

impl DrugComponent {
    pub fn generic_name(&self) -> &str { &self.generic_name }
    pub fn amount(&self) -> Rational64 { self.amount }
    pub fn unit(&self) -> &str { &self.unit }
}

impl DrugToDisplay {
    #[allow(unused)] pub fn index(&self) -> usize { self.index }
    pub fn drug(&self) -> &Drug { &self.drug }
    pub fn remaining_weeks(&self) -> Option<i64> { self.remaining_weeks }
    pub fn weeks_per_prescription(&self) -> Option<i64> { self.weeks_per_prescription }

    pub fn needs_replenishment(&self, min_weeks_per_prescription: &Option<i64>) -> bool {
        let mwpp = match min_weeks_per_prescription {
            Some(m) => *m,
            None => return false,
        };
        let rw = match self.remaining_weeks() {
            Some(w) => w,
            None => return false,
        };
        rw < mwpp
    }
}

impl DailyPills {
    pub fn morning(&self) -> u64 { self.morning }
    pub fn noon(&self) -> u64 { self.noon }
    pub fn evening(&self) -> u64 { self.evening }
    pub fn night(&self) -> u64 { self.night }

    pub fn increase_morning(&mut self, by: &Rational64) {
        if let Ok(numer) = u64::try_from(*by.ceil().numer()) {
            self.morning += numer;
        }
    }

    pub fn increase_noon(&mut self, by: &Rational64) {
        if let Ok(numer) = u64::try_from(*by.ceil().numer()) {
            self.noon += numer;
        }
    }

    pub fn increase_evening(&mut self, by: &Rational64) {
        if let Ok(numer) = u64::try_from(*by.ceil().numer()) {
            self.evening += numer;
        }
    }

    pub fn increase_night(&mut self, by: &Rational64) {
        if let Ok(numer) = u64::try_from(*by.ceil().numer()) {
            self.night += numer;
        }
    }
}
