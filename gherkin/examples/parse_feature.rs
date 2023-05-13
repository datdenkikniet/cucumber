use anyhow::Error;
use gherkin::Parser;

fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    args.next();

    let path = if let Some(path) = args.next() {
        path
    } else {
        return Err(anyhow::anyhow!("No path provided"));
    };

    let feature_data = std::fs::read_to_string(path)?;

    let feature_parsed = Parser::parse_feature(&feature_data).unwrap();

    println!("{:#?}", feature_parsed);

    Ok(())
}
