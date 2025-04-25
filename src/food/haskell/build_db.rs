use crate::food::{
    haskell::types::{perennial, HsId, Module},
    usda::{self, HaskellDB},
};
use std::path::Path;

pub fn build_db<P: AsRef<Path>>(root: P) {
    let edible = true;
    let inedible = false;
    let fooddb = usda::load_files(&[
        // Omit branded food, because there's so much of it it would overwhelm other data,
        // and is irrelevant - we don't care about specific brands.
        // data/source/FoodData_Central_branded_food_json_2024-04-18/brandedDownload.json
        // Omit survey food, because it's just random recipes.
        // data/source/FoodData_Central_survey_food_json_2022-10-28.json
        "data/source/FoodData_Central_sr_legacy_food_json_2018-04/FoodData_Central_sr_legacy_food_json_2021-10-28.json",
        "data/source/FoodData_Central_foundation_food_json_2024-04-18/foundationDownload.json",
    ]);

    Module::new(
        &["Crom", "FoodDB", "Ingredients", "Other", "Grains"],
        vec![
            fooddb.get_hs_food(
                "Rice, white, medium-grain, cooked, unenriched",
                HsId::new("rice"),
                "Rice, white, cooked",
                edible,
                perennial(),
            ),
            fooddb.get_hs_food(
                "Pasta, dry, unenriched",
                HsId::new("pasta"),
                "Pasta, dry",
                inedible,
                perennial(),
            ),
        ],
    )
    .write_source_file(&root);

    Module::new(
        &["Crom", "FoodDB", "Ingredients", "Other", "Vegetables"],
        vec![fooddb.get_hs_food(
            "Carrots, raw",
            HsId::new("carrots"),
            "Carrots, raw",
            edible,
            perennial(),
        )],
    )
    .write_source_file(&root);
}
