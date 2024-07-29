use std::error::Error;
use std::fs::File;
use std::env;
use clap::Parser;
use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    config: Option<String>,
    input: String,
    output: String,
}

#[derive(Deserialize)]
struct Config {
    line_start: Option<i32>,
    line_end: Option<i32>,
    replacement: Option<Vec<Replacement>>,
    fraction_digits: Option<u8>,
    selected: Option<Vec<Selected>>,
}

#[derive(Deserialize)]
struct Replacement {
    old: String,
    new: String,
}

#[derive(Deserialize)]
struct Selected {
    name: String,
    rename: Option<String>,
    fraction_digits: Option<u8>,
    replacement: Option<Vec<Replacement>>,
}

fn apply_replacements(value: &str, replacements: &[Replacement]) -> String {
    let mut new_value = value.to_string();
    for replacement in replacements {
        if new_value == replacement.old {
            new_value = replacement.new.clone();
            break;
        }
    }
    new_value
}

fn format_float(value: &str, fraction_digits: u8) -> String {
    match value.parse::<f64>() {
        Ok(num) => format!("{:.1$}", num, fraction_digits as usize),
        Err(_) => value.to_string(),
    }
}

fn select_columns(config: Config, input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().from_reader(file);

    let headers = rdr.headers()?.clone();
    let selected_columns: Vec<&Selected> = config.selected.as_ref().unwrap().iter().collect();
    let indices: Vec<usize> = selected_columns.iter()
        .map(|col| headers.iter().position(|h| h == col.name).unwrap())
        .collect();

    let mut wtr = csv::Writer::from_writer(File::create(output_path)?);
    let output_headers: Vec<String> = selected_columns.iter()
        .map(|col| col.rename.clone().unwrap_or_else(|| col.name.clone()))
        .collect();
    wtr.write_record(&output_headers)?;

    for (i, result) in rdr.records().enumerate() {
        let record = result?;
        let line_num = i as i32 + 1;
        if config.line_start.map_or(true, |start| line_num >= start) && config.line_end.map_or(true, |end| line_num <= end) {
            let selected: Vec<String> = indices.iter().enumerate().map(|(j, &i)| {
                let mut value = record[i].to_string();
                if let Some(global_replacements) = &config.replacement {
                    value = apply_replacements(&value, global_replacements);
                }
                if let Some(column_replacements) = &selected_columns[j].replacement {
                    value = apply_replacements(&value, column_replacements);
                }
                let fraction_digits = selected_columns[j].fraction_digits.or(config.fraction_digits);
                if let Some(digits) = fraction_digits {
                    value = format_float(&value, digits);
                }
                value
            }).collect();
            wtr.write_record(&selected)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

fn main()-> Result<(), Box<dyn Error>> {
        let args = Cli::parse();

    let config_path = args.config.or_else(|| env::var("FALCON_CONF").ok());
    let config: Config = if let Some(path) = config_path {
        let config_content = std::fs::read_to_string(path)?;
        toml::from_str(&config_content)?
    } else {
        return Err("Configuration file not specified and FALCON_CONF environment variable not set".into());
    };

    select_columns(config, &args.input, &args.output)?;

    Ok(())
}
