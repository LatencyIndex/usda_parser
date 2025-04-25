//! Module for interpreting food nutrient data from USDA json.

use serde_json as sjson;
use std::fmt;

pub struct Nutrient {
    pub name: String,
    pub amount: f64,
    pub unit: String,
}

pub struct Food {
    pub name: String,
    pub nutrients: Vec<Nutrient>,
}

impl fmt::Display for Nutrient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} {}", self.name, self.amount, self.unit)
    }
}

impl fmt::Display for Food {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nstr: String = self
            .nutrients
            .iter()
            .map(|s| format!("    {s}\n"))
            .collect::<Vec<String>>()
            .concat();
        write!(f, "{}:\n{}", self.name, nstr)
    }
}

fn as_nutrient(x: &sjson::Value) -> Option<Nutrient> {
    let amount = x["amount"].as_f64()?;
    let name = x["nutrient"]["name"].as_str()?.to_string();
    let unit = x["nutrient"]["unitName"].as_str()?.to_string();
    Some(Nutrient { name, amount, unit })
}

pub fn as_food(x: &sjson::Value) -> Option<Food> {
    let name = x["description"].as_str()?.to_string();
    let nutrients: Vec<Nutrient> = x["foodNutrients"]
        .as_array()?
        .iter()
        .filter_map(as_nutrient)
        .collect();
    Some(Food { name, nutrients })
}

fn contains_any(needles: &[&str], haystack: &str) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn is_essential_amino_acid(nutrient: &str) -> bool {
    let aminos = vec![
        "cysteine",
        "histidine",
        "isoleucine",
        "leucine",
        "lysine",
        "methionine",
        "phenylalanine",
        "threonine",
        "tryptophan",
        "tyrosine",
        "valine",
    ];
    contains_any(&aminos, &nutrient.to_lowercase())
}

pub fn has_amino_profile(food: &Food) -> bool {
    food.nutrients
        .iter()
        .any(|x| is_essential_amino_acid(&x.name))
}
