use crate::food::{haskell::serialization::AsHaskell, units::Quantity};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Month {
    Jan,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

pub fn perennial() -> BTreeSet<Month> {
    use Month::*;
    BTreeSet::from([Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec])
}

/// A Haskell value identifier.
#[derive(Clone)]
pub struct HsId {
    name: String,
}

impl HsId {
    /// Panics if the name is not a valid Haskell identifier.
    /// Must be non-empty, start with an ascii lowercase letter,
    /// and contain only ascii alphanumeric characters and underscores.
    pub fn new(name: &str) -> HsId {
        fn is_valid_char(c: char) -> bool {
            c.is_ascii_alphanumeric() || c == '_'
        }
        assert!(!name.is_empty());
        assert!(name.chars().next().is_some_and(|c| c.is_ascii_lowercase()));
        assert!(name.chars().all(is_valid_char));
        HsId {
            name: name.to_string(),
        }
    }
}

impl Display for HsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

pub struct Food {
    pub haskell_name: HsId, // Name of the haskell variable representing this food.
    pub human_name: String, // Name used when e.g. printing the food for an end-user.
    pub edible: bool,
    // The food database contains basic ingredients (e.g. rice, carrots, pasta..)
    // and not cooked dishes or recipes, so these fields are omitted/left empty.
    // ingredients :: M.Map Food MassFraction,
    // recipe :: T.Text,
    pub season: BTreeSet<Month>, // Which months of the year the food is in season.
    pub calories: Option<Quantity>,
    pub nutrients: BTreeMap<String, Quantity>,
}

pub struct Module {
    pub name: Vec<String>,
    pub foods: Vec<Food>,
}

impl Module {
    pub fn write_source_file<P: AsRef<Path>>(&self, root: P) {
        let path = root.as_ref().join(self.name.iter().collect::<PathBuf>());
        let dirs = path
            .parent()
            .expect("module name must have at least one element, but it had none");
        std::fs::create_dir_all(dirs).unwrap();
        std::fs::write(path.with_extension("hs"), self.as_haskell(0)).unwrap();
    }
}
