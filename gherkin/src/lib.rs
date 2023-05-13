mod data_table;
pub use data_table::DataTable;

mod parser;
pub use parser::Parser;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepType {
    Given,
    When,
    Then,
    And,
    But,
    Asterisk,
}

#[derive(Debug, Clone)]
pub enum StepData {
    DocString(String),
    DataTable(DataTable),
}

#[derive(Debug, Clone)]
pub struct Step {
    pub ty: StepType,
    pub description: String,
    pub data: Option<StepData>,
}

impl Step {
    pub fn new(ty: StepType, description: String, data: Option<StepData>) -> Self {
        Self {
            ty,
            description,
            data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone)]
pub struct ScenarioOutline {
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Vec<Step>,
    pub placeholders: Vec<String>,
    pub examples: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Feature {
    pub name: Option<String>,
    pub description: Option<String>,
    pub background: Vec<Step>,
    pub scenarios: Vec<Scenario>,
    pub scenario_outlines: Vec<ScenarioOutline>,
}
