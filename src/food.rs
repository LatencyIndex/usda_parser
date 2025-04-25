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
        match s {
            "IU"    => Ok(Unit::InternationalUnit),
            "g"     => Ok(Unit::Gram),
            "kJ"    => Ok(Unit::Kilojoule),
            "kcal"  => Ok(Unit::Kilocalorie),
            "mg"    => Ok(Unit::Milligram),
            "sp gr" => Ok(Unit::SpecificGravity),
            "µg"    => Ok(Unit::Microgram),
            u       => Ok(Unit::Other(u.to_string())),
        }
    }
}

impl fmt::Display for Unit {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Unit::InternationalUnit => "IU",
            Unit::Gram              => "g",
            Unit::Kilojoule         => "kJ",
            Unit::Kilocalorie       => "kcal",
            Unit::Milligram         => "mg",
            Unit::SpecificGravity   => "sp gr",
            Unit::Microgram         => "µg",
            Unit::Other(u)          =>  u,
        };
        write!(f, "{s}")
    }
}

#[derive(Clone)]
pub struct Quantity {
    pub amount: f64,
    pub unit: Unit,
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
