use std::{collections::HashSet, error::Error, fs::File, time::Instant};

use csv::ReaderBuilder;
use ryu::Buffer;

use crate::{Arguments, Column, Replacement};

use super::helper::print_time_cost;

fn apply_replacements(value: &str, replacements: &[Replacement]) -> String {
    replacements
        .iter()
        .find(|r| value == r.old)
        .map(|r| r.new.clone())
        .unwrap_or_else(|| value.to_owned())
}

fn format_float(value: &str, digits: usize, buffer: &mut Buffer) -> String {
    match value.parse::<f32>() {
        Ok(num) => {
            if digits == 0 {
                return (num.round() as i32).to_string();
            }
            let multiplier = 10.0f32.powi(digits as i32);
            buffer
                .format((num * multiplier).round() / multiplier)
                .to_owned()
        }
        Err(_) => value.to_owned(),
    }
}

fn get_col_fracdigits(
    data: &str,
    global_digits: Option<usize>,
    col_digits: Option<usize>,
) -> Option<usize> {
    if global_digits.is_none() && col_digits.is_none() {
        return None;
    }
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
) -> Option<Vec<Replacement>> {
    match global {
        Some(replacements) => {
            let mut set: HashSet<Replacement> = replacements.clone().into_iter().collect();
            if let Some(replacements) = this {
                set.extend(replacements.clone().into_iter());
            }
            let val = set.into_iter().collect();
            return Some(val);
        }
        None => match this {
            Some(replacements) => Some(replacements.clone()),
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
                name: selected.rename.clone().unwrap_or(selected.name.to_owned()),
                fraction_digits: selected.fraction_digits,
                replacement: merge_replacements(&config.replacement, &selected.replacement),
            }),
            None => return Err(format!("Column: '{}' not found", selected.name).into()),
        }
    }

    let mut wtr = csv::Writer::from_writer(File::create(output_path)?);
    let mut buffer = ryu::Buffer::new();
    let output_headers: Vec<String> = columns.iter().map(|col| col.name.to_owned()).collect();
    wtr.write_record(&output_headers)?;

    let mut record = csv::ByteRecord::new();

    let mut line_num = 1;
    while rdr.read_byte_record(&mut record)? {
        if line_num == 1 {
            for col in &mut columns {
                let col_data = &record[col.index];
                col.fraction_digits = get_col_fracdigits(
                    std::str::from_utf8(col_data)?,
                    config.fraction_digits,
                    col.fraction_digits,
                );
            }
        }
        if config.line_start.map_or(true, |start| line_num >= start)
            && config.line_end.map_or(true, |end| line_num <= end)
        {
            let selected: Vec<String> = columns
                .iter()
                .map(|col| {
                    let mut value = String::from_utf8(record[col.index].to_vec()).unwrap();
                    if let Some(replacements) = &col.replacement {
                        value = apply_replacements(&value, replacements);
                    }
                    if let Some(digits) = col.fraction_digits {
                        value = format_float(&value, digits, &mut buffer);
                    }
                    value
                })
                .collect();
            wtr.write_record(&selected)?;
        } else if config.line_end.map_or(false, |end| line_num > end) {
            break;
        }
        line_num += 1;
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

        let col_digits = None;
        let global_digits = None;
        let data = "-3.14159";
        assert_eq!(get_col_fracdigits(data, global_digits, col_digits), None);

        let col_digits = Some(2);
        let global_digits = Some(4);
        let data = "-3.14159";
        // check for column config override
        assert_eq!(
            get_col_fracdigits(data, global_digits, col_digits),
            col_digits
        );

        let data = "5";
        assert_eq!(get_col_fracdigits(data, global_digits, col_digits), None);
    }
}
