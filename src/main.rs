use riddle_of_protein::food::usda;
use std::{
    fs,
    io::{BufWriter, Write},
};

fn main() {
    // Load food data
    let foods = usda::food_from_files(&[
        // Omit branded food, because there's so much of it it would overwhelm other data,
        // and is irrelevant - we don't care about specific brands.
        // data/source/FoodData_Central_branded_food_json_2024-04-18/brandedDownload.json
        // Omit survey food, because it's just random recipes.
        // data/source/FoodData_Central_survey_food_json_2022-10-28.json
        "data/source/FoodData_Central_sr_legacy_food_json_2018-04/FoodData_Central_sr_legacy_food_json_2021-10-28.json",
        "data/source/FoodData_Central_foundation_food_json_2024-04-18/foundationDownload.json",
    ]);

    let mut out = BufWriter::new(fs::File::create_new("data/processed/test.txt").unwrap());
    for (name, _nutrients) in foods {
        out.write_all(format!("{}\n", name).as_bytes()).unwrap();
    }
}
