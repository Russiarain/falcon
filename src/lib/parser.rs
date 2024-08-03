use std::env::{self, args};

use crate::{Arguments, Config};

const ENV_VAR_NAME: &str = "FALCON_CONF";

const ENV_NOT_FOUND: &str = "env variable `FALCON_CONF` is not found";
const FAILED_TO_READ_CONFIG: &str = "Failed to read config toml file";
const FAILED_TO_PARSE_CONFIG: &str = "Failed to parse config toml file";

pub fn parse() -> Result<Option<Arguments>, &'static str> {
    let arg: Vec<String> = args().collect();
    if arg.len() == 1 {
        return Ok(None);
    }
    match arg.len() {
        2 => {
            let input = arg.get(1).unwrap();
            if !input.ends_with(".csv") {
                return Err("Input must be a csv file!");
            }
            let config = get_config(None)?;
            Ok(Some(Arguments {
                config,
                input: input.to_owned(),
                output: get_out_name(&input),
            }))
        }
        3 => {
            let input = arg.get(1).unwrap();
            if !input.ends_with(".csv") {
                return Err("Input must be a csv file!");
            }
            let output = arg.get(2).unwrap();
            if !output.ends_with(".csv") {
                return Err("Output must be a csv file!");
            }
            Ok(Some(Arguments {
                config: get_config(None)?,
                input: input.to_owned(),
                output: output.to_owned(),
            }))
        }
        4 => {
            let input = arg.get(1).unwrap();
            if !input.ends_with(".csv") {
                return Err("Input must be a csv file!");
            }
            let output = arg.get(2).unwrap();
            if !output.ends_with(".csv") {
                return Err("Output must be a csv file!");
            }
            Ok(Some(Arguments {
                config: get_config(Some(arg.get(3).unwrap().to_owned()))?,
                input: input.to_owned(),
                output: output.to_owned(),
            }))
        }
        _ => Err("Invalid usage"),
    }
}

fn get_config(conf: Option<String>) -> Result<Config, &'static str> {
    // TODO: maybe there's a simpler stype of propagating error here
    let conf_path = match conf {
        Some(path) => path,
        None => match env::var(ENV_VAR_NAME) {
            Ok(path) => path,
            _ => return Err(ENV_NOT_FOUND),
        },
    };
    let config_content = std::fs::read_to_string(&conf_path).map_err(|_| FAILED_TO_READ_CONFIG)?;
    let config: Config = toml::from_str(&config_content).map_err(|_| FAILED_TO_PARSE_CONFIG)?;
    Ok(config)
}

fn get_out_name(input: &str) -> String {
    let len = input.len();
    let mut outname = String::with_capacity(len + 2);
    outname.push_str(&input[..len - 4]);
    outname.push_str("_1.csv");
    outname
}
