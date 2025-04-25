//! Translate nutrients from how they are given by the USDA json, to how the Haskell code needs them.

use crate::food::rust;
use crate::food::units::{add_opt_qs, Quantity, Unit::Microgram};
use crate::lsl::algorithm::get_first;
use std::collections::BTreeMap;

pub fn derive_calories(nutrients: &rust::Nutrients) -> Option<Quantity> {
    // The Atwater specific factors value should be more accurate than
    // the general factor version. But if available, choose plain 'Energy' - presumably
    // it has been chosen as most accurate by the analysts.
    get_first(
        nutrients,
        [
            "Energy",
            "Energy (Atwater General Factors)",
            "Energy (Atwater Specific Factors)",
        ]
        .into_iter(),
    )
    .cloned()
}

pub fn derive_nutrients(nutrients: &rust::Nutrients) -> BTreeMap<String, Quantity> {
    // Helper functions
    let get = |key: &str| -> Option<Quantity> { nutrients.get(key).cloned() };
    let priority =
        |keys: &[&str]| -> Option<Quantity> { get_first(nutrients, keys.iter().copied()).cloned() };
    let weighted = |key: &str, weight: f64| -> Option<Quantity> { get(key).map(|x| x * weight) };
    // Weighted sum. Missing keys counted as zero. If no keys are found, returns None.
    // Panics if any quantity has incompatible units.
    let weighted_sum = |weighted_keys: &[(&str, f64)]| -> Option<Quantity> {
        let qs = weighted_keys.iter().filter_map(|(k, w)| weighted(k, *w));
        qs.clone().next().and(Some(qs.sum())) // Returns None if qs is empty.
    };
    // Same as weighted_sum, but all weights are 1.
    let sum = |keys: &[&str]| -> Option<Quantity> {
        let weighted_keys: Vec<(&str, f64)> = keys.iter().map(|k| (*k, 1.0)).collect();
        weighted_sum(weighted_keys.as_slice())
    };

    // Attributes on expressions are experimental, so we make this a statement.
    #[rustfmt::skip]
    // The only nutrients given in International Units (IU) are:
        // "Vitamin A, IU"
        // "Vitamin D (D2 + D3), International Units"
    // NLEA: Nutrition labeling and education act of 1990
    let nutrients_haskell: BTreeMap<String, Quantity> = BTreeMap::from_iter([
        // By summation is a newer and more accurate method than by difference.
        ("Carbohydrates",       priority(&[
                                    "Carbohydrate, by summation",
                                    "Carbohydrate, by difference",])),
        ("Sugars",              priority(&[
                                    "Sugars, Total",
                                    "Total Sugars",
                                    "Sugars, total including NLEA",])),
        ("Fat",                 priority(&[
                                    "Total fat (NLEA)",
                                    "Total lipid (fat)",])),
        ("Fiber",               priority(&[
                                    "Fiber, total dietary",
                                    "Total dietary fiber (AOAC 2011.25)"])
                                .or(sum(&[
                                    "Fiber, insoluble",
                                    "Fiber, soluble"]))
                                .or(sum(&[
                                    "High Molecular Weight Dietary Fiber (HMWDF)",
                                    "Low Molecular Weight Dietary Fiber (LMWDF)"]))),
        ("Protein",             get("Protein")), // Total, not necessarily complete.
        // Has to be calculated form amino acid makeup.
        // However, it is not additive, so it is more meaningful to calculate it for an entire
        // meal/day, than for a single food.
        ("CompleteProtein",     None),
        ("Element_B",           get("Boron, B")),
        ("Element_Ca",          get("Calcium, Ca")),
        ("Element_Co",          get("Cobalt, Co")),
        ("Element_Cu",          get("Copper, Cu")),
        ("Element_F",           get("Fluoride, F")),
        ("Element_Fe",          get("Iron, Fe")),
        ("Element_I",           get("Iodine, I")),
        ("Element_K",           get("Potassium, K")),
        ("Element_Mg",          get("Magnesium, Mg")),
        ("Element_Mn",          get("Manganese, Mn")),
        ("Element_Mo",          get("Molybdenum, Mo")),
        ("Element_Na",          get("Sodium, Na")),
        ("Element_P",           get("Phosphorus, P")),
        ("Element_S",           get("Sulfur, S")),
        ("Element_Se",          get("Selenium, Se")),
        ("Element_Zn",          get("Zinc, Zn")),
        ("Amino_Cysteine",      get("Cysteine")),
        ("Amino_Histidine",     get("Histidine")),
        ("Amino_Isoleucine",    get("Isoleucine")),
        ("Amino_Leucine",       get("Leucine")),
        ("Amino_Lysine",        get("Lysine")),
        ("Amino_Methionine",    get("Methionine")),
        ("Amino_Phenylalanine", get("Phenylalanine")),
        ("Amino_Threonine",     get("Threonine")),
        ("Amino_Tryptophan",    get("Tryptophan")),
        ("Amino_Tyrosine",      get("Tyrosine")),
        ("Amino_Valine",        get("Valine")),
        ("Omega3",              sum(&[
                                    "PUFA 18:3 n-3 c,c,c (ALA)", // Alpha-linolenic acid
                                    "PUFA 20:3 n-3",
                                    "PUFA 20:5 n-3 (EPA)",
                                    "PUFA 22:5 n-3 (DPA)",
                                    "PUFA 22:6 n-3 (DHA)",
                                    "PUFA 2:5 n-3 (EPA)",])), // Probably a typo for 20:5
        ("Omega6",              sum(&[
                                    "PUFA 18:2 n-6 c,c",
                                    "PUFA 18:3 n-6 c,c,c",
                                    "PUFA 20:2 n-6 c,c",
                                    "PUFA 20:3 n-6",
                                    "PUFA 2:4 n-6",])), // Probably a typo for 2x:4
        ("Omega7",              None), // Not in USDA database.
        ("Omega9",              sum(&[
                                    "MUFA 22:1 n-9", // Erucic acid
                                    "PUFA 20:3 n-9",])), // Mead acid
        // Dosage equivalents at https://en.wikipedia.org/wiki/Vitamin_A#Units_of_measurement
        // Dietary recommendations given in RAE (retinol activity equivalent)
        ("Vitamin_A",           get("Vitamin A, IU") // Always given in IU units, as the name promises. If not, we panic.
                                    // 3.33 IU of vitamin A is equivalent to 1 microgram of RAE
                                    .map(|q| q.from_IU(Quantity{amount: 1.0/3.33, unit: Microgram}))
                                    .or(get("Vitamin A, RAE"))
                                    .or(weighted_sum(&[
                                        ("Retinol", 1.0),
                                        ("Cryptoxanthin, beta", 1.0 / 24.0),]))),
        ("Vitamin_B1",          get("Thiamin")),
        ("Vitamin_B2",          get("Riboflavin")),
        ("Vitamin_B3",          get("Niacin")),
        ("Vitamin_B5",          get("Pantothenic acid")),
        ("Vitamin_B6",          get("Vitamin B-6")),
        ("Vitamin_B7",          get("Biotin")),
        ("Vitamin_B9",          get("Folate, DFE") // DFE: Dietary folate equivalent.
                                    .or(add_opt_qs(
                                        priority(&[
                                            "Folate, total",
                                            "Folate, food"]), // Folate presumably does not include folic acid.
                                        weighted("Folic acid", 1.7)))), // 1 microgram of folic acid is equivalent to 1.7 microgram DFE.
        ("Vitamin_B12",         sum(&[
                                    "Vitamin B-12",
                                    "Vitamin B-12, added",])),
        ("Vitamin_C",           get("Vitamin C, total ascorbic acid")),
        // Vitamins D2 and D3 are biologically equivalent, others are non-essential.
        ("Vitamin_D",           get("Vitamin D (D2 + D3)")
                                    .or( // For vitamin D, 1 microgram = 40 IU
                                        get("Vitamin D (D2 + D3), International Units")
                                        .map(|q| q.from_IU(Quantity{amount: 1.0/40.0, unit: Microgram})))
                                    .or(sum(&[
                                        "Vitamin D2 (ergocalciferol)",
                                        "Vitamin D3 (cholecalciferol)",]))),
        ("Vitamin_E",           sum(&[
                                    "Tocopherol, beta",
                                    "Tocopherol, delta",
                                    "Tocopherol, gamma",
                                    "Tocotrienol, alpha",
                                    "Tocotrienol, beta",
                                    "Tocotrienol, delta",
                                    "Tocotrienol, gamma",
                                    "Vitamin E (alpha-tocopherol)",
                                    // "added" vitamins are separate from other vitamin analysis, and don't overlap.
                                    // Meaning they should be added to the total.
                                    "Vitamin E, added",])),
        ("Vitamin_K",           sum(&[
                                    // Dihydrophylloquinone is associated with lower bone density
                                    // (which vitamin K is supposed to help)
                                    // and I found no information on how effective it is as a vitamin K,
                                    // so as a precaution it is excluded from total vitamin K.
                                    // "Vitamin K (Dihydrophylloquinone)",
                                    // Vitamins K1 (phylloquinone) & K2 (menaquinone) are considered equivalent.
                                    "Vitamin K (Menaquinone-4)",
                                    "Vitamin K (phylloquinone)",])),
        ("Cholesterol",         get("Cholesterol")),
        // In the USDA database, if any form of choline is given, then the choline total is also given.
        // So we concern ourselves only with the total, and ignore the others.
        ("Choline",             get("Choline, total")),
        ("Coenzyme_Q10",        None), // Not in USDA database.
        ("Collagen",            None), // Not in USDA database.
        ("Creatine",            None), // Not in USDA database.
    ].into_iter().filter_map(|(s,optq)| {
        optq.map(|q| (s.to_string(), q))
    }));
    nutrients_haskell
}
