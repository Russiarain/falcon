use serde::Deserialize;

pub mod lib {
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

#[derive(Deserialize)]
pub struct Replacement {
    old: String,
    new: String,
}

#[derive(Deserialize)]
pub struct Selected {
    name: String,
    rename: Option<String>,
    fraction_digits: Option<usize>,
    replacement: Option<Vec<Replacement>>,
}

pub struct Arguments {
    config: Config,
    input: String,
    output: String,
}