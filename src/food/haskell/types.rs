use crate::food::{haskell::serialization::AsHaskell, units::Quantity, usda::Nutrients};
use std::collections::BTreeSet;
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

pub struct Food {
    pub haskell_name: String, // Name of the haskell variable representing this food.
    pub human_name: String,   // Name used when e.g. printing the food for an end-user.
    pub edible: bool,
    // The food database contains basic ingredients (e.g. rice, carrots, pasta..)
    // and not cooked dishes or recipes, so these fields are omitted/left empty.
    // ingredients :: M.Map Food MassFraction,
    // recipe :: T.Text,
    pub season: BTreeSet<Month>, // Which months of the year the food is in season.
    pub calories: Option<Quantity>,
    pub nutrients: Nutrients,
}

pub struct Module {
    pub name: Vec<String>,
    pub foods: Vec<Food>,
}

impl Module {
    pub fn to_source_file<P: AsRef<Path>>(&self, root: P) {
        let path = root.as_ref().join(self.name.iter().collect::<PathBuf>());
        let dirs = path
            .parent()
            .expect("Module name must have at least one element, but it had none.");
        std::fs::create_dir_all(dirs).unwrap();
        std::fs::write(path.with_extension("hs"), self.as_haskell(0)).unwrap();
    }
}
