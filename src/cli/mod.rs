use std::ops::Not;
use std::path::PathBuf;

use clap::{command, Parser, ValueEnum};

use crate::prelude::{Deserialize, Serialize};

#[derive(Debug, Parser, Clone)]
#[command(
    name = "parse_csv_rs",
    about = "A CLI tool to parse a CSV file and filter out rows based on a set of criteria.",
    author = "Blake B.",
    long_about = "\n
    This CLI tool is designed to parse a CSV file and filter out columns & rows based on a set of criteria.
    The criteria are defined via a configuration file, or overrideable via CLI arguments.\n
    If Command Line Interface (CLI) arguments are provided, they will always override the values in the configuration file.",
    version = clap::crate_version!(),
    arg_required_else_help = false,
    styles=get_styles()
)]
#[rustfmt::skip]
pub struct Cli {
    /// The source CSV file to parse.
    #[arg(name = "source", index = 1, help = "The source CSV file to parse - overrides the source file in the config file file if provided.", required = false, value_hint = clap::ValueHint::FilePath)]
    pub source: Option<PathBuf>,

    /// The configuration file to use - overrides the default configuration file.
    #[arg(name = "config_file", short = 'c', long = "config", help = "The configuration file to use - overrides the default configuration file.", required = false, value_hint = clap::ValueHint::FilePath)]
    pub config_file: Option<PathBuf>,
    
    /// The output type to use.
    #[arg(name = "output_type", short = 't', long = "output_type", help = "The output type to use.", required = false, value_hint = clap::ValueHint::Other, value_enum,)]
    pub output_type: Option<OutputType>,

    #[arg(name = "output_path", short = 'o', long = "output", help = "The output file path to use.", required = false, value_hint = clap::ValueHint::FilePath)]
    pub output_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, ValueEnum)]
pub enum OutputType {
    #[value(name = "stdout", alias = "stdout", alias = "Stdout", alias = "0")]
    #[serde(rename = "stdout")]
    Stdout,

    #[value(name = "csv", alias = "csv", alias = "Csv", alias = "1")]
    #[serde(rename = "csv")]
    Csv,
}

impl Default for OutputType {
    fn default() -> Self {
        OutputType::Stdout
    }
}

impl PartialEq for OutputType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (OutputType::Stdout, OutputType::Stdout) => true,
            (OutputType::Csv, OutputType::Csv) => true,
            _ => false,
        }
    }
}

impl Eq for OutputType {}

impl Not for OutputType {
    type Output = OutputType;

    fn not(self) -> Self::Output {
        match self {
            OutputType::Stdout => OutputType::Csv,
            OutputType::Csv => OutputType::Stdout,
        }
    }
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
