use serde_json as json;
use std::{
    fmt, fs,
    io::{BufWriter, Write},
    path::Path,
};

pub struct Nutrient {
    name: String,
    amount: f64,
    unit: String,
}

pub struct Food {
    name: String,
    nutrients: Vec<Nutrient>,
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

fn as_nutrient(x: &json::Value) -> Option<Nutrient> {
    let amount = x["amount"].as_f64()?;
    let name = x["nutrient"]["name"].as_str()?.to_string();
    let unit = x["nutrient"]["unitName"].as_str()?.to_string();
    Some(Nutrient { name, amount, unit })
}

pub fn as_food(x: &json::Value) -> Option<Food> {
    let name = x["description"].as_str()?.to_string();
    let nutrients: Vec<Nutrient> = x["foodNutrients"]
        .as_array()?
        .iter()
        .filter_map(as_nutrient)
        .collect();
    Some(Food { name, nutrients })
}

fn has_microprotein(food: &Food) -> bool {
    food.nutrients
        .iter()
        .any(|x| x.name.to_lowercase().contains("leucine"))
}

// Parse a USDA nutrition json file into Food items, then filter and write them to output file.
pub fn transmute_usda<FPath, T, P>(inp: FPath, out: FPath, tf: T, pred: P)
where
    FPath: AsRef<Path>,
    T: Fn(&Food) -> String,
    P: Fn(&Food) -> bool,
{
    let inp = fs::read_to_string(inp).expect("Couldn't read input file");
    let mut out = BufWriter::new(fs::File::create_new(out).expect("Couldn't create output file"));

    // The stream parser consumes exactly one complete json object at a time.
    // It does not parse any commas/brackets delimiting/containing the json objects.
    // So we manually ignore those delimiters from the input string.

    // That is, if the string is "[<json_object>,<json_object>,<json_object>]",
    // we limit it to "<json_object>,<json_object>,<json_object>",
    // parse the first json object, skip the comma, and repeat.

    // Skip to the [ that starts the outer-most json array that contains the values:
    let mut i = inp.find('[').expect("Couldn't find [");

    loop {
        let mut stream = json::Deserializer::from_str(&inp[i + 1..]).into_iter::<json::Value>();
        match stream.next() {
            // Keep parsing as long as input can be parsed to json, even if it can't be converted to Food:
            Some(Ok(val)) => {
                if let Some(food) = as_food(&val) {
                    if pred(&food) {
                        out.write(tf(&food).as_bytes())
                            .expect("Couldn't write to output file.");
                    }
                }
            }
            _ => break,
        };
        i += stream.byte_offset(); // Skip already parsed bytes.
        i += inp[i..].find(',').unwrap_or(0); // Skip comma delimiter after json value, if any.
    }
}
