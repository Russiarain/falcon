use std::{collections::HashSet, hash::{Hash, Hasher}};

use serde::Deserialize;

pub mod lib {
    pub mod helper;
    pub mod parser;
    pub mod runner;
}

#[derive(Deserialize)]
pub struct Config {
    line_start: Option<i32>,
    line_end: Option<i32>,
    replacement: Option<Vec<Replacement>>,
    fraction_digits: Option<usize>,
    selected: Option<Vec<Selected>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Replacement {
    old: String,
    new: String,
}

impl PartialEq for Replacement {
    fn eq(&self, other: &Self) -> bool {
        self.old == other.old
    }
}

impl Eq for Replacement {}

impl Hash for Replacement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.old.hash(state);
    }
}

#[derive(Deserialize)]
pub struct Selected {
    name: String,
    rename: Option<String>,
    fraction_digits: Option<usize>,
    replacement: Option<Vec<Replacement>>,
}

pub struct Column {
    index: usize,
    name: String,
    fraction_digits: Option<usize>,
    replacement: Option<HashSet<Replacement>>,
}

pub struct Arguments {
    config: Config,
    input: String,
    output: String,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::Replacement;

    #[test]
    fn replacement_set() {
        let replacements = [
            Replacement {
                old: String::from("a"),
                new: String::from("x"),
            },
            Replacement {
                old: String::from("b"),
                new: String::from("y"),
            },
            Replacement {
                old: String::from("a"),
                new: String::from("z"),
            },
        ];
        let mut set = HashSet::new();
        for r in replacements {
            if set.contains(&r) {
                set.replace(r);
            } else {
                set.insert(r);
            }
        }

        assert!(set.len() == 2);
        assert_eq!(
            set.get(&Replacement {
                old: String::from("a"),
                new: String::from("x")
            }),
            Some(&Replacement {
                old: String::from("a"),
                new: String::from("z")
            })
        );
    }
}
