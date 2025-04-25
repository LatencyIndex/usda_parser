//! Module for interpreting food nutrient data from USDA json.

pub mod units;

use serde_json as sj;
use std::{collections::BTreeMap, fmt};
use units::{Quantity, Unit};

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

fn as_nutrient(x: &sj::Value) -> Option<(String, Quantity)> {
    let amount = x["amount"].as_f64()?;
    let name = x["nutrient"]["name"].as_str()?.trim().to_string();
    let unit = x["nutrient"]["unitName"].as_str()?.trim().parse().ok()?;
    Some((name, Quantity { amount, unit }))
}

pub fn as_food(x: &sj::Value) -> Option<Food> {
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
