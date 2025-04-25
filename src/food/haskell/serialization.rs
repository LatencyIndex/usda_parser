// use crate::food::rust::Food as RustFood;
use crate::food::haskell::types::{Food, Module, Month};
use crate::food::units::{Quantity, Unit};
use crate::lsl::numeric::round_to_n_significant;
use std::collections::{BTreeMap, BTreeSet};

const TAB_WIDTH: usize = 4;

/// Serialize to Haskell source code.
pub trait AsHaskell {
    fn as_haskell(&self, indent: usize) -> String;
}

// Deref-coercion is insufficient, because it is applied only at the start.
// E.g. &(&i32, &i32) will be coerced to (&i32, &i32), but no more,
// and the compiler will find no AsHaskell impl for &i32 if the impl below were omitted.
impl<T: AsHaskell + ?Sized> AsHaskell for &T {
    fn as_haskell(&self, indent: usize) -> String {
        (*self).as_haskell(indent)
    }
}

impl<T: AsHaskell, U: AsHaskell> AsHaskell for (T, U) {
    fn as_haskell(&self, indent: usize) -> String {
        format!(
            "({}, {})",
            self.0.as_haskell(indent),
            self.1.as_haskell(indent)
        )
    }
}

impl<T: AsHaskell> AsHaskell for Option<T> {
    fn as_haskell(&self, indent: usize) -> String {
        match self {
            Some(x) => format!("Just ({})", x.as_haskell(indent)),
            None => "Nothing".to_string(),
        }
    }
}

impl<K: AsHaskell, V: AsHaskell> AsHaskell for BTreeMap<K, V> {
    fn as_haskell(&self, indent: usize) -> String {
        // Doesn't need indent, because it's on the same line as its caller. E.g.
        // <prior indent> themap = M.fromList [
        //                   item1,
        //                   item2,
        //                   item3
        //                   ]
        let mut s = "M.fromList [".to_string();
        let mut iter = self.iter().peekable();
        let reindent = indent + TAB_WIDTH;

        while let Some(item) = iter.next() {
            let item = item.as_haskell(reindent);
            let sep = if iter.peek().is_some() {
                ",".to_string()
            } else {
                format!("\n{:reindent$}]", " ")
            };
            s.push_str(&format!("\n{:reindent$}{item}{sep}", " "));
        }
        s
    }
}

impl AsHaskell for bool {
    fn as_haskell(&self, _indent: usize) -> String {
        match self {
            true => "True",
            false => "False",
        }
        .to_string()
    }
}

impl AsHaskell for str {
    fn as_haskell(&self, _indent: usize) -> String {
        // Probably does not handle quotes or newlines inside the string well,
        // but we don't need that.
        format!("\"{self}\"")
    }
}

impl AsHaskell for String {
    fn as_haskell(&self, indent: usize) -> String {
        self.as_str().as_haskell(indent)
    }
}

impl AsHaskell for Unit {
    fn as_haskell(&self, _indent: usize) -> String {
        use Unit::*;
        // Though the units are absolute, they are given in the context of content per 100g of food.
        // Since the haskell code uses mass fractions directly, we simply serialize to _per_100g,
        // and skip any conversion.
        match self {
            Gram => "g_per_100g",
            Milligram => "mg_per_100g",
            Microgram => "ug_per_100g",
            Kilojoule => "kJ_per_100g",
            Kilocalorie => "kcal_per_100g",
            _ => panic!("unit {self} cannot be serialized to Haskell"),
        }
        .to_string()
    }
}

impl AsHaskell for Quantity {
    fn as_haskell(&self, indent: usize) -> String {
        format!(
            "{} *. {}",
            // 3 digits is enough precision for dietary purposes.
            round_to_n_significant(self.amount, 3),
            self.unit.as_haskell(indent)
        )
    }
}

impl AsHaskell for Month {
    fn as_haskell(&self, _indent: usize) -> String {
        use Month::*;
        match self {
            Jan => "Jan",
            Feb => "Feb",
            Mar => "Mar",
            Apr => "Apr",
            May => "May",
            Jun => "Jun",
            Jul => "Jul",
            Aug => "Aug",
            Sep => "Sep",
            Oct => "Oct",
            Nov => "Nov",
            Dec => "Dec",
        }
        .to_string()
    }
}

impl AsHaskell for BTreeSet<Month> {
    fn as_haskell(&self, indent: usize) -> String {
        let has_every_month = self.len() == 12;
        if has_every_month {
            "perennial".to_string()
        } else {
            let list: String = self
                .iter()
                .map(|x| x.as_haskell(indent))
                .collect::<Vec<String>>()
                .join(", ");
            format!("S.fromList [{list}]")
        }
    }
}

impl AsHaskell for Food {
    fn as_haskell(&self, indent: usize) -> String {
        let reindent = indent + TAB_WIDTH;
        let tab = " ".repeat(indent);
        let retab = " ".repeat(reindent);

        let haskell_name = &self.haskell_name;
        let human_name = self.human_name.as_haskell(reindent);
        let edible = self.edible.as_haskell(reindent);
        let season = self.season.as_haskell(reindent);
        let calories = self.calories.as_haskell(reindent);
        let nutrients = self.nutrients.as_haskell(reindent);

        // No newline at the end, so that if a food is the last item in a file,
        // the file does not end with a blank line. We want to avoid this because
        // automatic whitespace trimming could delete that blank line, causing diff noise.
        format!(
            "{tab}{haskell_name} :: Food
{tab}{haskell_name} = Food {{
{retab}name = {human_name},
{retab}edible = {edible},
{retab}ingredients = M.empty,
{retab}recipe = \"\",
{retab}season = {season},
{retab}calories = {calories},
{retab}nutrients = {nutrients}
{tab}}}"
        )
    }
}

impl AsHaskell for Module {
    fn as_haskell(&self, _indent: usize) -> String {
        // A module is its own file, so it always starts with 0 indentation.
        let tab = " ".repeat(TAB_WIDTH);
        let indent = 0;
        let module_name = self.name.join(".");
        let exports: String = self
            .foods
            .iter()
            .map(|food| format!("{tab}{}", food.haskell_name))
            .collect::<Vec<String>>()
            .join(",\n");
        let foods: String = self
            .foods
            .iter()
            .map(|food| food.as_haskell(indent))
            .collect::<Vec<String>>()
            .join("\n\n");

        format!(
            "-- Auto-generated file; do not edit.

module {module_name} (
{exports}
) where

import Crom.Food
import qualified Crom.Calendar as C
import qualified Data.Map as M
import qualified Data.Set as S

{foods}"
        )
    }
}
