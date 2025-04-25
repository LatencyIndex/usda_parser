use riddle_of_protein as rp;
use std::{
    fs,
    io::{BufWriter, Write},
};

fn main() {
    // Load all the food data as String (its json representation is too big for RAM)
    let inps: Vec<String> = [
        "data/source/FoodData_Central_foundation_food_json_2024-04-18.json",
        "data/source/FoodData_Central_sr_legacy_food_json_2021-10-28.json",
    ]
    .iter()
    .map(|path| fs::read_to_string(path).expect(&format!("couldn't read {path}")))
    .collect();

    // Iterate over its contained food items:
    let foods = inps
        .iter()
        .filter_map(|s| rp::USDAIter::new(s))
        .flatten() // Chain iterators over each file
        .filter_map(|json| rp::as_food(&json));

    let mut out = BufWriter::new(fs::File::create_new("data/processed/test.txt").unwrap());
    for food in foods {
        out.write(format!("{}\n", food.name).as_bytes()).unwrap();
    }
}
