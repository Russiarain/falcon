use std::{collections::HashMap, f64, fs::File, time::Instant};

use anyhow::{Result, anyhow};
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

fn f2str_with_digits(value: f64, digits: usize, buffer: &mut Buffer) -> String {
    if digits == 0 {
        return (value.round() as i32).to_string();
    }
    let multiplier = 10.0f64.powi(digits as i32);
    buffer
        .format((value * multiplier).round() / multiplier)
        .to_owned()
}

fn format_with_digits(value: &str, digits: usize, buffer: &mut Buffer) -> String {
    match value.parse::<f64>() {
        Ok(num) => f2str_with_digits(num, digits, buffer),
        Err(_) => value.to_owned(),
    }
}

fn remove_invalid_fraction_digits(col: &mut Column, col_data: &str, global_digits: Option<usize>) {
    match col.manipulate {
        Manipulate::Transform(_) => {
            col.fraction_digits = col.fraction_digits.or(global_digits).or(Some(2));
        }
        _ => {
            col.fraction_digits = if global_digits.is_none() && col.fraction_digits.is_none() {
                None
            } else {
                match col_data.parse::<i32>() {
                    Ok(_) => None,
                    Err(_) => match col_data.parse::<f32>() {
                        Ok(_) => col.fraction_digits.or(global_digits),
                        Err(_) => None,
                    },
                }
            };
        }
    }
}

fn remove_invalid_transform(col: &mut Column, col_data: &str) {
    if let Manipulate::Transform(_) = col.manipulate {
        if let Err(_) = col_data.parse::<f64>() {
            col.manipulate = Manipulate::None;
        }
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
    let mut transforms = HashMap::new();
    match &config.selected {
        Some(selections) => {
            for selected in selections {
                if selected.replacement.is_some() && selected.transform.is_some() {
                    return Err(anyhow!(
                        "replacement and transform can not be set at the same time for column: {}",
                        selected.name
                    ));
                }
                match headers.iter().position(|h| h == selected.name) {
                    Some(idx) => {
                        let output_name =
                            selected.rename.clone().unwrap_or(selected.name.to_owned());
                        columns.push(Column {
                            index: idx,
                            name: output_name.clone(),
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
                                                match expr.clone().bind("x") {
                                                    Ok(f)=>{
                                                        transforms.insert(output_name, f);
                                                    },
                                                    Err(_)=>return Err(anyhow!(
                                                        "Failed to bind template variable of transform fcn for column: {}",
                                                        selected.name
                                                    ))
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
                        });
                    }
                    None => return Err(anyhow!("Column: '{}' not found", selected.name)),
                }
            }
        }
        None => {
            let mut idx = 0;
            for header in headers.iter() {
                columns.push(Column {
                    index: idx,
                    name: header.to_owned(),
                    fraction_digits: config.fraction_digits,
                    manipulate: Manipulate::None,
                });
                idx += 1;
            }
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
                if col.manipulate != Manipulate::None && col_data.is_empty() {
                    return Err(anyhow!(
                        "First line of column: '{}' must not be empty!",
                        col.name
                    ));
                }
                let col_data = std::str::from_utf8(col_data)?;
                remove_invalid_transform(col, col_data);
                remove_invalid_fraction_digits(col, col_data, config.fraction_digits);
            }
        }
        if config.line_start.map_or(true, |start| line_num >= start)
            && config.line_end.map_or(true, |end| line_num <= end)
        {
            let selected: Vec<String> = columns
                .iter()
                .map(|col| {
                    let mut value = String::from_utf8(record[col.index].to_vec()).unwrap();
                    if value.is_empty() {
                        return value;
                    }
                    match &col.manipulate {
                        Manipulate::Replace(replacements) => {
                            value = apply_replacements(&value, replacements);
                        }
                        Manipulate::Transform(_) => {
                            let transform = transforms.get(&col.name).unwrap();
                            return f2str_with_digits(
                                transform(value.parse::<f64>().unwrap()),
                                col.fraction_digits.unwrap(),
                                &mut buffer,
                            );
                        }
                        _ => (),
                    }

                    if let Some(digits) = col.fraction_digits {
                        value = format_with_digits(&value, digits, &mut buffer);
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
