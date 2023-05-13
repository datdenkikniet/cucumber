use std::{fs::File, io::Read};

use anyhow::Error;
use gherkin::Parser;

fn parse_feature(name: &str, mut file: File) -> Result<(), Error> {
    let mut str = String::with_capacity(131072);
    file.read_to_string(&mut str)?;
    let _ = Parser::parse_feature(&str).expect(&format!("Failed for {:?}", name));

    // println!("{:#?}", feature_parsed);

    Ok(())
}

fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let mut args = std::env::args();
    args.next();

    let path = if let Some(path) = args.next() {
        path
    } else {
        return Err(anyhow::anyhow!("No path provided"));
    };

    let file = File::open(&path)?;
    parse_feature(&path, file)?;

    Ok(())
}
