use std::{fs::File, io::Read};

use anyhow::Error;
use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// Print the contents of the parsed feature.
    #[clap(long, short)]
    pub print: bool,

    pub file: String,
}

fn parse_feature(name: &str, mut file: File, print: bool) -> Result<(), Error> {
    let mut str = String::with_capacity(131072);
    file.read_to_string(&mut str)?;

    let feature = gherkin::Parser::parse_feature(&str).expect(&format!("Failed for {:?}", name));

    if print {
        println!("{feature:#?}");
    }

    let scenario_count = feature.scenarios().count();
    let computed_scenario_count = feature.total_scenario_count();
    assert_eq!(scenario_count, computed_scenario_count);
    Ok(())
}

fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    let file = File::open(&cli.file)?;
    parse_feature(&cli.file, file, cli.print)?;

    Ok(())
}
