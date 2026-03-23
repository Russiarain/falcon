use colored::Colorize;

const VERSION: &str = "0.4.0";
const PRJ_URL: &str = "https://github.com/Russiarain/falcon";

pub fn print_help() {
    println!("{}\n","CSV toolkit for column/row selecting, cell content replacing/transforming and number rounding".bold());
    println!("{}", "Usage:".green());
    println!("{} {}", "falcon".bright_cyan(), "[input].csv".bright_blue());
    println!("    process [input].csv using config from env, create [input]_1.csv");
    println!(
        "{} {}",
        "falcon".bright_cyan(),
        "[input].csv [output].csv".bright_blue()
    );
    println!("    process [input].csv using config from env, create [output].csv");
    println!(
        "{} {}",
        "falcon".bright_cyan(),
        "[input].csv [output].csv [conf.toml]".bright_blue()
    );
    println!("    process [input].csv using config from [conf.toml], create [output].csv");
    println!(
        "\n{} {} {}",
        "use",
        "set FALCON_CONF=your_config_file.toml".yellow(),
        "to specify a configuration file from env."
    );
    println!(
        "\nCurrent version: {}, see {} for more information.",
        VERSION.magenta(),
        PRJ_URL
    );
}

pub fn print_error(err: &str) {
    eprintln!("{}", err.red());
}

pub fn print_time_cost(n: u128) {
    println!("{} in {} ms", "Finished".green(), n);
}
