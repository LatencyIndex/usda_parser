//! Module for interpreting food nutrient data from USDA json.

use serde_json as sjson;
use std::{collections::BTreeMap, fmt, str::FromStr};

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
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.unit)
    }
}

pub struct Food {
    pub name: String,
    pub nutrients: BTreeMap<String, Quantity>,
}

impl fmt::Display for Food {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nstr: String = self
            .nutrients
            .iter()
            .map(|(name, quantity)| format!("    {name}: {quantity}\n"))
            .collect::<Vec<String>>()
            .concat();
        write!(f, "{}:\n{}", self.name, nstr)
    }
}

fn as_nutrient(x: &sjson::Value) -> Option<(String, Quantity)> {
    let amount = x["amount"].as_f64()?;
    let name = x["nutrient"]["name"].as_str()?.trim().to_string();
    let unit = x["nutrient"]["unitName"].as_str()?.trim().parse().ok()?;
    Some((name, Quantity { amount, unit }))
}

pub fn as_food(x: &sjson::Value) -> Option<Food> {
    let name = x["description"].as_str()?.trim().to_string();
    let mut nutrients = BTreeMap::new();
    for (k, v) in x["foodNutrients"]
        .as_array()?
        .iter()
        .filter_map(as_nutrient)
    {
        // The only 'nutrient' to ever repeat itself in the food DB is Energy.
        // It is usually listed twice, in kJ and in kcal.
        // We prefer the kcal version, so only insert the kJ version if kcal is absent.
        if !(k == "Energy" && v.unit == Unit::Kilojoule && nutrients.contains_key(&k)) {
            nutrients.insert(k, v);
        }
    }
    Some(Food { name, nutrients })
}
