use std::{fmt, str::FromStr};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unit {
    InternationalUnit,
    Gram,
    Kilojoule,
    Kilocalorie,
    Milligram,
    SpecificGravity, // Only used for one liquid, can ignore.
    Microgram,
    Other(String), // Needed for robustness, e.g. when exploring new datasets.
}

impl FromStr for Unit {
    type Err = String;
    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Unit::*;
        match s {
            "IU"    => Ok(InternationalUnit),
            "g"     => Ok(Gram),
            "kJ"    => Ok(Kilojoule),
            "kcal"  => Ok(Kilocalorie),
            "mg"    => Ok(Milligram),
            "sp gr" => Ok(SpecificGravity),
            "µg"    => Ok(Microgram),
            u       => Ok(Other(u.to_string())),
        }
    }
}

impl fmt::Display for Unit {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Unit::*;
        let s = match self {
            InternationalUnit => "IU",
            Gram              => "g",
            Kilojoule         => "kJ",
            Kilocalorie       => "kcal",
            Milligram         => "mg",
            SpecificGravity   => "sp gr",
            Microgram         => "µg",
            Other(u)          =>  u,
        };
        write!(f, "{s}")
    }
}

#[derive(Clone)]
pub struct Quantity {
    pub amount: f64,
    pub unit: Unit,
}

impl Quantity {
    #[rustfmt::skip]
    pub fn convert_to(&self, unit: &Unit) -> Quantity {
        use Unit::*;
        match (&self.unit, unit) {
            (Gram, Milligram)        => Quantity {amount: self.amount * 1e+3,  unit: Milligram},
            (Gram, Microgram)        => Quantity {amount: self.amount * 1e+6,  unit: Microgram},
            (Milligram, Gram)        => Quantity {amount: self.amount * 1e-3,  unit: Gram},
            (Milligram, Microgram)   => Quantity {amount: self.amount * 1e+3,  unit: Microgram},
            (Microgram, Gram)        => Quantity {amount: self.amount * 1e-6,  unit: Gram},
            (Microgram, Milligram)   => Quantity {amount: self.amount * 1e-3,  unit: Milligram},
            (Kilojoule, Kilocalorie) => Quantity {amount: self.amount * 0.239, unit: Kilocalorie},
            (Kilocalorie, Kilojoule) => Quantity {amount: self.amount * 4.184, unit: Kilojoule},
            (u, w) =>   if u == w {
                          self.clone()
                        } else if self.amount == 0.0 {
                            Quantity {amount: self.amount, unit: unit.clone()}
                        } else {
                            panic!("cannot convert units {u} to {w}")
                        },
        }
    }

    #[allow(non_snake_case)]
    pub fn from_IU(&self, amount_per_IU: Quantity) -> Quantity {
        match self.unit {
            Unit::InternationalUnit => amount_per_IU * self.amount,
            _ => panic!("called from_IU on non-IU unit {}", self.unit),
        }
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.unit)
    }
}

impl std::default::Default for Quantity {
    fn default() -> Self {
        Quantity {
            amount: 0.0,
            unit: Unit::Gram,
        }
    }
}

impl std::ops::Mul<f64> for Quantity {
    type Output = Quantity;
    fn mul(self, x: f64) -> Quantity {
        Quantity {
            amount: self.amount * x,
            ..self
        }
    }
}

impl std::ops::Div<f64> for Quantity {
    type Output = Quantity;
    fn div(self, x: f64) -> Quantity {
        self * (1.0 / x)
    }
}

impl std::ops::Add<Quantity> for Quantity {
    type Output = Quantity;
    fn add(self, rhs: Quantity) -> Quantity {
        if self.amount == 0.0 {
            rhs
        } else if rhs.amount == 0.0 {
            self
        } else {
            Quantity {
                amount: self.amount + rhs.convert_to(&self.unit).amount,
                unit: self.unit,
            }
        }
    }
}

impl std::ops::AddAssign for Quantity {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

impl std::iter::Sum for Quantity {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc: Quantity = Quantity::default();
        for q in iter {
            acc += q;
        }
        acc
    }
}
