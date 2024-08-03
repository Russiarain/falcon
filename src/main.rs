use std::error::Error;
use std::process;

use falcon::lib::parser;
use falcon::lib::runner;

fn main() -> Result<(), Box<dyn Error>> {
    let args = parser::parse().unwrap_or_else(|err|{
        eprintln!("{err}");
        process::exit(64);
    });

    match args {
        Some(arg) => {
            if let Err(msg) = runner::run(arg) {
                eprintln!("Error:\n{}",msg);
            }
            
        }
        None => println!("helper"),
    }

    Ok(())
}
