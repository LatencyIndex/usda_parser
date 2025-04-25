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
    pub fn convert_to(&self, unit: &Unit) -> Option<Quantity> {
        use Unit::*;
        match (&self.unit, unit) {
            (u, w) if u == w         => Some(self.clone()),
            (Gram, Milligram)        => Some(Quantity {amount: self.amount * 1e+3,  unit: Milligram}),
            (Gram, Microgram)        => Some(Quantity {amount: self.amount * 1e+6,  unit: Microgram}),
            (Milligram, Gram)        => Some(Quantity {amount: self.amount * 1e-3,  unit: Gram}),
            (Milligram, Microgram)   => Some(Quantity {amount: self.amount * 1e+3,  unit: Microgram}),
            (Microgram, Gram)        => Some(Quantity {amount: self.amount * 1e-6,  unit: Gram}),
            (Microgram, Milligram)   => Some(Quantity {amount: self.amount * 1e-3,  unit: Milligram}),
            (Kilojoule, Kilocalorie) => Some(Quantity {amount: self.amount * 0.239, unit: Kilocalorie}),
            (Kilocalorie, Kilojoule) => Some(Quantity {amount: self.amount * 4.184, unit: Kilojoule}),
            _ => None,
        }
    }

    #[allow(non_snake_case)]
    pub fn from_IU(&self, amount_per_IU: Quantity) -> Option<Quantity> {
        match self.unit {
            Unit::InternationalUnit => Some(amount_per_IU * self.amount),
            _ => None,
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
    type Output = Option<Quantity>;
    fn add(self, other: Quantity) -> Option<Quantity> {
        other.convert_to(&self.unit).map(|x| Quantity {
            amount: x.amount + self.amount,
            unit: x.unit,
        })
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.unit)
    }
}
