//! Module for exploring the USDA food nutrient data.

use crate::food;
use std::collections::BTreeSet;

fn contains_any(needles: &[&str], haystack: &str) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn is_essential_amino_acid(nutrient: &str) -> bool {
    const AMINOS: [&str; 11] = [
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
    contains_any(&AMINOS, &nutrient.to_lowercase())
}

pub fn has_amino_profile(food: &food::Food) -> bool {
    food.nutrients.keys().any(|x| is_essential_amino_acid(x))
}

pub fn unique_nutrients(foods: impl Iterator<Item = food::Food>) -> BTreeSet<String> {
    foods
        .flat_map(|f| f.nutrients.keys().cloned().collect::<Vec<String>>())
        .collect()
}

pub fn unique_units(foods: impl Iterator<Item = food::Food>) -> BTreeSet<food::Unit> {
    foods
        .flat_map(|f| {
            f.nutrients
                .values()
                .map(|v| v.unit.clone())
                .collect::<Vec<food::Unit>>()
        })
        .collect()
}
