//! Module for converting food nutrient data from USDA json to a rust representation.

use crate::food::units::{Quantity, Unit};
use serde_json as sj;
use std::collections::BTreeMap;

pub type Nutrients = BTreeMap<String, Quantity>;

fn as_nutrient(x: &sj::Value) -> Option<(String, Quantity)> {
    let amount = x["amount"].as_f64()?;
    let name = x["nutrient"]["name"].as_str()?.trim().to_string();
    let unit = x["nutrient"]["unitName"].as_str()?.trim().parse().ok()?;
    Some((name, Quantity { amount, unit }))
}

/// Return the name and nutrient content of the food represented by the json.
pub fn to_food(x: &sj::Value) -> Option<(String, Nutrients)> {
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
    Some((name, nutrients))
}
