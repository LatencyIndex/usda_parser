use riddle_of_protein::{food, usda};
use std::{
    fs,
    io::{BufWriter, Write},
};

fn main() {
    // Default locations to where the zipped files unzip.
    let inps: Vec<String> = [
        // Omit branded food, because there's so much of it it would overwhelm other data,
        // and is irrelevant - we don't care about specific brands.
        // "data/source/FoodData_Central_branded_food_json_2024-04-18/brandedDownload.json",
        // Omit survey food, because it's just random recipes.
        // "data/source/FoodData_Central_survey_food_json_2022-10-28.json",
        "data/source/FoodData_Central_foundation_food_json_2024-04-18/foundationDownload.json",
        "data/source/FoodData_Central_sr_legacy_food_json_2018-04/FoodData_Central_sr_legacy_food_json_2021-10-28.json",
    ] // Load all the food data as String (its json representation is too big for RAM)
    .iter()
    .map(|path| fs::read_to_string(path).unwrap_or_else(|_| panic!("couldn't read {path}")))
    .collect();

    // Iterate over its contained food items:
    let foods = inps
        .iter()
        // Create and chain iterators in one step.
        // Unwrap because failure to create an iterator should panic
        .flat_map(|s| usda::USDAIter::new(s).unwrap())
        .filter_map(|json_val| food::as_food(&json_val));

    let mut out = BufWriter::new(fs::File::create_new("data/processed/test.txt").unwrap());
    for food in foods {
        out.write_all(format!("{}\n", food.name).as_bytes())
            .unwrap();
    }
}
