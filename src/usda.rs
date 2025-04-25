//! Module for parsing the USDA food data json files.

use serde_json as sj;

/// Discard the string outside of and including the outer-most pair of []
fn trim_to_brackets(s: &str) -> Option<&str> {
    s.split_once('[')
        .and_then(|(_begin, rest)| rest.rsplit_once(']'))
        .map(|(mid, _tail)| mid)
}

/// Iterates over json values in USDA json files, parsing incrementally.
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
    type Item = sj::Value;

    fn next(&mut self) -> Option<Self::Item> {
        // The stream parser consumes exactly one complete json object at a time.
        // It does not parse any commas/brackets delimiting/containing the json objects.
        // So we manually ignore those delimiters from the input string.

        // That is, if the string is "<json_value>,<json_value>,<json_value>",
        // parse the first json object, skip the comma, and repeat.
        let mut json_stream = sj::Deserializer::from_str(self.inp).into_iter::<sj::Value>();
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
                Some(val)
            }
            // When conversion to json fails, assume there are no more json values left to parse.
            _ => None,
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
