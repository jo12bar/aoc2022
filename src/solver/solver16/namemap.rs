use std::{fmt, hash::Hash};

use super::parse::{Name, MAX_NAME};

#[derive(Clone)]
pub struct NameMap<T> {
    values: Vec<Option<T>>,
}

impl<T> NameMap<T> {
    pub fn new() -> Self {
        Self {
            values: std::iter::repeat_with(|| None).take(MAX_NAME).collect(),
        }
    }

    pub fn get(&self, name: Name) -> Option<&T> {
        self.values[name.as_usize()].as_ref()
    }

    pub fn get_mut(&mut self, name: Name) -> Option<&mut T> {
        self.values[name.as_usize()].as_mut()
    }

    pub fn insert(&mut self, name: Name, value: T) {
        self.values[name.as_usize()] = Some(value);
    }

    pub fn contains(&self, name: Name) -> bool {
        self.values[name.as_usize()].is_some()
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.keys().any(|k| other.contains(k))
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        !self.overlaps(other) && !other.overlaps(self)
    }

    pub fn is_empty(&self) -> bool {
        self.values.iter().all(|v| v.is_none())
    }

    pub fn iter(&self) -> impl Iterator<Item = (Name, &T)> {
        self.values
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|v| (Name::from_usize(i), v)))
    }

    pub fn keys(&self) -> impl Iterator<Item = Name> + '_ {
        self.values
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|_| Name::from_usize(i)))
    }
}

impl<T> Default for NameMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Debug for NameMap<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> FromIterator<(Name, T)> for NameMap<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Name, T)>,
    {
        let mut map = Self::new();

        for (name, value) in iter {
            map.insert(name, value);
        }

        map
    }
}

impl<T> PartialEq for NameMap<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl<T> Eq for NameMap<T> where T: Eq {}

impl<T> Hash for NameMap<T>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.values.hash(state);
    }
}
