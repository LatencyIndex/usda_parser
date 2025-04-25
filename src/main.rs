mod usda_parser;

fn main() {
    // let inp = "data/source/FoodData_Central_foundation_food_json_2024-04-18.json";
    let inp = "data/source/FoodData_Central_sr_legacy_food_json_2021-10-28.json";
    let out = "data/processed/legacy_food.txt";

    usda_parser::transmute_usda(inp, out, |x| x.to_string(), |_| true);
}
