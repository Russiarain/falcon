use std::{collections::HashSet, error::Error, fs::File, time::Instant};

use csv::ReaderBuilder;

use crate::{Arguments, Column, Replacement};

use super::helper::print_time_cost;

fn apply_replacements(value: &str, replacements: &HashSet<Replacement>) -> String {
    let mut new_value = value.to_owned();
    for replacement in replacements {
        if new_value == replacement.old {
            new_value = replacement.new.clone();
            break;
        }
    }
    new_value
}

fn format_float(value: &str, fraction_digits: usize) -> String {
    match value.parse::<f32>() {
        Ok(num) => format!("{:.1$}", num, fraction_digits),
        Err(_) => value.to_owned(),
    }
}

fn get_col_fracdigits(
    data: &str,
    global_digits: Option<usize>,
    col_digits: Option<usize>,
) -> Option<usize> {
    match data.parse::<i32>() {
        Ok(_) => None,
        Err(_) => match data.parse::<f32>() {
            Ok(_) => col_digits.or_else(|| global_digits),
            Err(_) => None,
        },
    }
}

fn merge_replacements(
    global: &Option<Vec<Replacement>>,
    this: &Option<Vec<Replacement>>,
) -> Option<HashSet<Replacement>> {
    match global {
        Some(replacements) => {
            let mut set = HashSet::from_iter(replacements.clone().into_iter());
            if let Some(replacements) = this {
                set.extend(replacements.clone().into_iter());
            }
            return Some(set);
        }
        None => match this {
            Some(replacements) => Some(HashSet::from_iter(replacements.clone().into_iter())),
            None => None,
        },
    }
}

pub fn run(arg: Arguments) -> Result<(), Box<dyn Error>> {
    let Arguments {
        config,
        input: input_path,
        output: output_path,
    } = arg;
    let start_time = Instant::now();
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().from_reader(file);

    let headers = rdr.headers()?.clone();
    let mut columns: Vec<Column> = Vec::new();
    for selected in config.selected.unwrap().iter() {
        match headers.iter().position(|h| h == selected.name) {
            Some(idx) => columns.push(Column {
                index: idx,
                name: selected
                    .rename
                    .to_owned()
                    .unwrap_or(selected.name.to_owned()),
                fraction_digits: selected.fraction_digits,
                replacement: merge_replacements(&config.replacement, &selected.replacement),
            }),
            None => return Err(format!("Column: '{}' not found", selected.name).into()),
        }
    }

    let mut wtr = csv::Writer::from_writer(File::create(output_path)?);
    let output_headers: Vec<String> = columns.iter().map(|col| col.name.to_owned()).collect();
    wtr.write_record(&output_headers)?;

    for (i, result) in rdr.records().enumerate() {
        let record = result?;
        let line_num = i as i32 + 1;
        if line_num == 1 {
            for col in &mut columns {
                let col_data = &record[col.index];
                col.fraction_digits =
                    get_col_fracdigits(col_data, config.fraction_digits, col.fraction_digits);
            }
        }
        if config.line_start.map_or(true, |start| line_num >= start)
            && config.line_end.map_or(true, |end| line_num <= end)
        {
            let selected: Vec<String> = columns
                .iter()
                .map(|col| {
                    let mut value = record[col.index].to_owned();
                    if let Some(replacements) = &col.replacement {
                        value = apply_replacements(&value, replacements);
                    }
                    if let Some(digits) = col.fraction_digits {
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

#[cfg(test)]
mod tests {
    use crate::lib::runner::get_col_fracdigits;

    #[test]
    fn col_fracdigits_test() {
        let col_digits = None;
        let global_digits = Some(4);

        let data = "3";
        assert_eq!(get_col_fracdigits(data, global_digits, col_digits), None);
        let data = "-3.14159";
        assert_eq!(
            get_col_fracdigits(data, global_digits, col_digits),
            global_digits
        );
        let data = "hello";
        assert_eq!(get_col_fracdigits(data, global_digits, col_digits), None);
    }
}
