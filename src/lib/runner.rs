use std::{error::Error, fs::File, time::Instant};

use csv::ReaderBuilder;

use crate::{Arguments, Replacement, Selected};

use super::helper::print_time_cost;

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

fn format_float(value: &str, fraction_digits: usize) -> String {
    match value.parse::<f64>() {
        Ok(num) => format!("{:.1$}", num, fraction_digits),
        Err(_) => value.to_string(),
    }
}

pub fn run(arg:Arguments) -> Result<(), Box<dyn Error>> {
    let Arguments { config, input:input_path, output:output_path } = arg;
    let start_time = Instant::now();
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().from_reader(file);

    let headers = rdr.headers()?.clone();
    let selected_columns: Vec<&Selected> = config.selected.as_ref().unwrap().iter().collect();
    let indices: Vec<usize> = selected_columns
        .iter()
        .map(|col| headers.iter().position(|h| h == col.name).unwrap())
        .collect();

    let mut wtr = csv::Writer::from_writer(File::create(output_path)?);
    let output_headers: Vec<String> = selected_columns
        .iter()
        .map(|col| col.rename.clone().unwrap_or_else(|| col.name.clone()))
        .collect();
    wtr.write_record(&output_headers)?;

    for (i, result) in rdr.records().enumerate() {
        let record = result?;
        let line_num = i as i32 + 1;
        if config.line_start.map_or(true, |start| line_num >= start)
            && config.line_end.map_or(true, |end| line_num <= end)
        {
            let selected: Vec<String> = indices
                .iter()
                .enumerate()
                .map(|(j, &i)| {
                    let mut value = record[i].to_string();
                    if let Some(global_replacements) = &config.replacement {
                        value = apply_replacements(&value, global_replacements);
                    }
                    if let Some(column_replacements) = &selected_columns[j].replacement {
                        value = apply_replacements(&value, column_replacements);
                    }
                    let fraction_digits = selected_columns[j]
                        .fraction_digits
                        .or(config.fraction_digits);
                    if let Some(digits) = fraction_digits {
                        value = format_float(&value, digits);
                    }
                    value
                })
                .collect();
            wtr.write_record(&selected)?;
        }
    }

    wtr.flush()?;

    print_time_cost(start_time.elapsed().as_millis());
    Ok(())
}

