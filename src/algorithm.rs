//! Generic algorithms and helper functions.

use std::{borrow::Borrow, collections::BTreeMap};

pub fn contains_any(needles: &[&str], haystack: &str) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

/// Return the first key that is found, or None if none are found.
pub fn get_first<'a, K, V, Q>(
    m: &'a BTreeMap<K, V>,
    keys: impl Iterator<Item = &'a Q>,
) -> Option<&'a V>
where
    Q: Ord + ?Sized + 'a,
    K: Borrow<Q> + Ord,
{
    keys.filter_map(|k| m.get(k)).next()
}
