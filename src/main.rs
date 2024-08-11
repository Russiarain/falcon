use std::error::Error;
use std::process;

use falcon::lib::helper;
use falcon::lib::parser;
use falcon::lib::runner;
use nu_ansi_term::enable_ansi_support;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = enable_ansi_support();

    let args = parser::parse().unwrap_or_else(|err| {
        helper::print_error(&err);
        process::exit(64);
    });

    match args {
        Some(arg) => {
            if let Err(msg) = runner::run(arg) {
                helper::print_error(&msg.to_string());
            }
        }
        None => helper::print_help(),
    }

    Ok(())
}
