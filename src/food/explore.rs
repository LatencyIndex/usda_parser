//! Module for exploring the USDA food nutrient data.

use crate::{
    food::{rust::Nutrients, units::Unit},
    lsl::algorithm::contains_any,
};
use std::collections::BTreeSet;

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

pub fn has_amino_profile(nutrients: &Nutrients) -> bool {
    nutrients.keys().any(|x| is_essential_amino_acid(x))
}

pub fn unique_nutrients(nutrients: impl Iterator<Item = Nutrients>) -> BTreeSet<String> {
    nutrients
        .flat_map(|x| x.keys().cloned().collect::<Vec<String>>())
        .collect()
}

pub fn unique_units(foods: impl Iterator<Item = Nutrients>) -> BTreeSet<Unit> {
    foods
        .flat_map(|f| f.values().map(|v| v.unit.clone()).collect::<Vec<Unit>>())
        .collect()
}
