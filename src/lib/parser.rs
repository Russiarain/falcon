use std::{
    env::{self, args},
    fs::read_to_string,
    path::Path,
};

use crate::{Arguments, Config};

const ENV_VAR_NAME: &str = "FALCON_CONF";

const ENV_NOT_FOUND: &str = "env variable `FALCON_CONF` is not found";
const FAILED_TO_READ_CONFIG: &str = "Failed to read config toml file";
const FAILED_TO_PARSE_CONFIG: &str = "Failed to parse config toml file";

pub fn parse() -> Result<Option<Arguments>, String> {
    let arg: Vec<String> = args().collect();
    let (input, output, conf) = match arg.len() {
        1 => return Ok(None),
        2 => {
            let input = arg.get(1).unwrap();
            (input.clone(), get_out_name(&input), None)
        }
        3 => (
            arg.get(1).unwrap().clone(),
            arg.get(2).unwrap().clone(),
            None,
        ),
        4 => (
            arg.get(1).unwrap().clone(),
            arg.get(2).unwrap().clone(),
            Some(arg.get(3).unwrap().to_owned()),
        ),
        _ => return Err("Invalid usage".to_string()),
    };
    check_extension_name_and_existence(&input, "csv")?;
    let mut toml_path = String::new();
    match conf {
        Some(confpath) => {
            check_extension_name_and_existence(&confpath, "toml")?;
            toml_path.push_str(&confpath);
        }
        None => match env::var(ENV_VAR_NAME) {
            Ok(path) => {
                check_extension_name_and_existence(&path, "toml")?;
                toml_path.push_str(&path);
            }
            Err(_) => return Err(ENV_NOT_FOUND.to_string()),
        },
    }
    Ok(Some(Arguments {
        config: get_config(toml_path)?,
        input,
        output,
    }))
}

fn check_extension_name_and_existence(path: &str, extension: &str) -> Result<(), String> {
    if !path.ends_with(extension) {
        return Err(format!(
            "The input '{}' must be a {} file!",
            path, extension
        ));
    }
    match Path::new(path).try_exists() {
        Ok(exist) => {
            if !exist {
                return Err(format!("File not found: {}", path));
            }
        }
        Err(_) => return Err(format!("No permission to read file: {}", path)),
    }
    Ok(())
}

fn get_config(conf: String) -> Result<Config, &'static str> {
    let config_content = read_to_string(&conf).map_err(|_| FAILED_TO_READ_CONFIG)?;
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
