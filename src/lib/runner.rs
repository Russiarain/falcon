use std::{f64, fs::File, time::Instant};

use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use ryu::Buffer;

use crate::{Arguments, Column, Manipulate, Replacement};

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

pub fn run(arg: Arguments) -> Result<()> {
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
        if selected.replacement.is_some() && selected.transform.is_some() {
            return Err(anyhow!(
                "replacement and transform can not be set at the same time for column: {}",
                selected.name
            ));
        }
        match headers.iter().position(|h| h == selected.name) {
            Some(idx) => columns.push(Column {
                index: idx,
                name: selected.rename.clone().unwrap_or(selected.name.to_owned()),
                fraction_digits: selected.fraction_digits,
                manipulate: {
                    if selected.replacement.is_none() && selected.transform.is_none() {
                        Manipulate::None
                    } else if selected.replacement.is_some() {
                        Manipulate::Replace(selected.unique_replacements().unwrap())
                    } else {
                        Manipulate::Transform({
                            match selected.transform.as_ref().unwrap().parse::<meval::Expr>() {
                                Ok(expr) => {
                                    if let Err(_) = expr.clone().bind("x") {
                                        return Err(anyhow!(
                                            "Failed to bind template variable of transform fcn for column: {}",
                                            selected.name
                                        ));
                                    }
                                    expr
                                }
                                Err(_) => {
                                    return Err(anyhow!(
                                        "Failed to parse transform fcn for column: {}",
                                        selected.name
                                    ))
                                }
                            }
                        })
                    }
                },
            }),
            None => return Err(anyhow!("Column: '{}' not found", selected.name)),
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
                let col_data = std::str::from_utf8(col_data)?;
                col.fraction_digits =
                    get_col_fracdigits(col_data, config.fraction_digits, col.fraction_digits);
                if let Manipulate::Transform(_) = col.manipulate {
                    if let Err(_) = col_data.parse::<f64>() {
                        col.manipulate = Manipulate::None;
                    }
                }
            }
        }
        if config.line_start.map_or(true, |start| line_num >= start)
            && config.line_end.map_or(true, |end| line_num <= end)
        {
            let selected: Vec<String> = columns
                .iter()
                .map(|col| {
                    let mut value = String::from_utf8(record[col.index].to_vec()).unwrap();
                    match &col.manipulate {
                        Manipulate::Replace(replacements) => {
                            value = apply_replacements(&value, replacements);
                        }
                        Manipulate::Transform(expr) => {
                            let transform = expr.clone().bind("x").unwrap();
                            value = transform(value.parse::<f64>().unwrap()).to_string();
                        }
                        _ => (),
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
