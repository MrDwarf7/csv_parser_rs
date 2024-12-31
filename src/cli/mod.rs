use std::path::PathBuf;

use clap::{command, Parser};

#[derive(Debug, Parser, Clone)]
#[command(
    name = "parse_csv.rs",
    about = "A CLI tool to parse a CSV file and filter out rows based on a set of criteria.",
    author = "Blake B.",
    long_about = "\n
    This CLI tool is designed to parse a CSV file and filter out rows based on a set of criteria.
    The criteria are defined in a configuration file, which is JSON file.

    If Command Line Interface (CLI) arguments are provided, they will override the values in the configuration file.
    ",
    version = std::env!("CARGO_PKG_VERSION"),
    arg_required_else_help = false,
    styles=get_styles()
)]
#[rustfmt::skip]
pub struct Cli {
    /// The source CSV file to parse.
    #[arg(index = 1, help = "The source CSV file to parse - overrides the source file in the config file file if provided.", value_hint = clap::ValueHint::FilePath, required = false)]
    pub source: Option<PathBuf>,

    /// The configuration file to use - overrides the default configuration file.
    #[arg(short = 'c', long = "config", help = "The configuration file to use - overrides the default configuration file.", value_hint = clap::ValueHint::FilePath, required = false)]
    pub config_file: Option<PathBuf>,
    
    /// The output type to use.
    #[arg(value_enum, short = 'o', long = "output_type", help = "The output type to use.", required = false)]
    pub output: Option<crate::config::Output>,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
}

pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))), // When a command is inc. This is the tag collor for 'Usage:'
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Blue))), // Main headers in the help menu (e.g. Arguments, Options)
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::BrightWhite))), // Strings for args etc. { -t, --total }
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red)))
                .effects(anstyle::Effects::ITALIC),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan))),
        )
        .placeholder(anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))))
}
