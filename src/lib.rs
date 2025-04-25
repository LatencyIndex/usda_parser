use serde_json as json;
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

fn contains_any(needles: &Vec<&str>, haystack: &str) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn is_essential_amino_acid(nutrient: &str) -> bool {
    let aminos = vec![
        "cysteine",
        "cystine",
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

/// Discard the string outside of and including the outer-most pair of []
fn trim_to_brackets(s: &str) -> Option<&str> {
    s.split_once('[')
        .and_then(|(_begin, rest)| rest.rsplit_once(']'))
        .map(|(mid, _tail)| mid)
}

pub struct USDAIter<'a> {
    inp: &'a str,
    pub bytes_parsed: usize,
}

impl<'a> USDAIter<'a> {
    pub fn new(inp: &'a str) -> Option<USDAIter<'a>> {
        // Some of the datasets are too big to parse in one go, so the parsing is done incrementally.
        // However, the json is composed of one top-level node, in the following form:
        // {"nameOfFoodDB": [<json_value>,<json_value>,<json_value>,..]}
        // This is a problem for the stream parser, because even though it consumes one json value at a time,
        // the entire file is a single (recursive) json value. That is why we manually descend one level
        // by manually trimming the string slice to inside the outer-most []:
        //      <json_value>,<json_value>,<json_value>,..<json_value>
        // and treat the json values as separate, as if they are not part of a json array.
        trim_to_brackets(inp).map(|trimmed| USDAIter {
            inp: trimmed,
            bytes_parsed: inp.len() - trimmed.len(),
        })
    }
}

impl<'a> Iterator for USDAIter<'a> {
    type Item = json::Value;

    fn next(&mut self) -> Option<Self::Item> {
        // The stream parser consumes exactly one complete json object at a time.
        // It does not parse any commas/brackets delimiting/containing the json objects.
        // So we manually ignore those delimiters from the input string.

        // That is, if the string is "<json_value>,<json_value>,<json_value>",
        // parse the first json object, skip the comma, and repeat.
        let mut json_stream = json::Deserializer::from_str(self.inp).into_iter::<json::Value>();
        match json_stream.next() {
            // Keep parsing as long as input can be parsed to json.
            Some(Ok(val)) => {
                let bytes_old = self.inp.len();

                // Skip already parsed bytes:
                self.inp = &self.inp[json_stream.byte_offset()..];
                // Skip comma delimiter after json value, if any:
                self.inp = self.inp.trim_start();
                self.inp = self.inp.strip_prefix(',').unwrap_or(self.inp);

                let bytes_new = self.inp.len();
                self.bytes_parsed += bytes_old - bytes_new;
                return Some(val);
            }
            // When conversion to json fails, assume there are no more json values left to parse.
            _ => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim() {
        assert_eq!(Some("mid"), trim_to_brackets("head [mid] tail"));
    }
}
