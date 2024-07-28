use std::{
    error::Error,
    fs::{read_to_string, File},
    process,
};

use csv::StringRecord;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    line_start: Option<i32>,
    line_end: Option<i32>,
    replacement: Option<Vec<Replacement>>,
    fraction_digits: Option<u8>,
    selected: Option<Vec<Selected>>,
}

#[derive(Debug, Deserialize)]
struct Replacement {
    old: String,
    new: String,
}

#[derive(Debug, Deserialize)]
struct Selected {
    name: String,
    rename: Option<String>,
    fraction_digits: Option<u8>,
    replacement: Option<Vec<Replacement>>,
}

struct Column {
    // index in original data
    index: usize,
    // output name
    name: String,
    // Some(value) if this column is floating point number
    // None for integer and other strings
    fraction_digits: Option<u8>,
    // global replacement will always be the first
    replacement: Option<Vec<Replacement>>,
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let conf = read_to_string("conf.toml").unwrap();
    let conf: Config = toml::from_str(&conf).unwrap();

    let file = File::open("data.csv")?;
    let mut record = StringRecord::new();
    let mut rdr = csv::Reader::from_reader(file);
    let header = rdr.headers().unwrap();
    for selected in conf.selected.unwrap() {
        let idx = header.iter().position(|val| *val == selected.name);
        match idx {
            Some(val) => println!("get {} at index {}", selected.name, val),
            None => println!("{} is not found", selected.name),
        }
    }
    let _ = rdr.read_record(&mut record);
    println!("{:?}", &record);

    Ok(())
}
