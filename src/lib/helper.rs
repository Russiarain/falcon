use nu_ansi_term::Color::{Green, LightBlue, LightCyan, Red};
use nu_ansi_term::Style;
use nu_ansi_term::{AnsiString, AnsiStrings};

const VERSION: &str = "0.1.0";
const PRJ_URL: &str = "https://github.com/Russiarain/falcon";

pub fn print_help() {
    let content: &[AnsiString<'static>] = &[
        Style::new().bold().paint(
            "CSV toolkit for column/row selecting, cell content replacing and number rounding\n\n",
        ),
        Green.paint("Usage:\n"),
        LightCyan.paint("falcon"),
        LightBlue.paint(" [input].csv\n"),
        Style::default()
            .paint("    process [input].csv using config from env, create [input]_1.csv\n"),
        LightCyan.paint("falcon"),
        LightBlue.paint(" [input].csv [output].csv\n"),
        Style::default()
            .paint("    process [input].csv using config from env, create [output].csv\n"),
        LightCyan.paint("falcon"),
        LightBlue.paint(" [input].csv [output].csv [conf.toml]\n"),
        Style::default()
            .paint("    process [input].csv using config from [conf.toml], create [output].csv\n"),
    ];
    println!("{}", AnsiStrings(content));
    println!(
        "\nCurrent version: {}, see {} for more information.",
        VERSION, PRJ_URL
    );
}

pub fn print_error(err: &str) {
    eprintln!("{}", Red.paint(err));
}
