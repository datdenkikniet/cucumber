mod lex;
pub use lex::{lex, Error as LexError, ErrorKind as LexErrorKind, Token};

mod parse;
pub use parse::{parse_feature, Error};

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    pub language: String,
    pub name: Option<String>,
    pub freeform_text: Option<String>,
    pub background_and: Vec<Prompt>,
    pub background_but: Vec<Prompt>,
    pub contents: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Background(String),
    Example(Example),
    ScenarioOutline(ScenarioOutline),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Example {
    pub name: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepType {
    Given,
    When,
    Then,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub ty: StepType,
    pub ands: Vec<Prompt>,
    pub buts: Vec<Prompt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Prompt {
    Bare { prompt: String },
    WithInput { prompt: String, input: StepInput },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepInput {
    String(String),
    Table(Table),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioOutline {
    pub steps: Vec<Step>,
    pub parameters: Vec<String>,
    pub examples: Vec<Vec<String>>,
}
