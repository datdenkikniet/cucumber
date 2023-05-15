mod data_table;
pub use data_table::DataTable;

mod parser;
pub use parser::Parser;

mod scenario_outline;
pub use scenario_outline::ScenarioOutline;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepType {
    Given,
    When,
    Then,
    And,
    But,
    Asterisk,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepData {
    DocString(String),
    DataTable(DataTable),
}

impl StepData {
    pub fn replace(&mut self, from: &str, to: &str) {
        match self {
            StepData::DocString(value) => {
                *value = value.replace(from, to);
            }
            StepData::DataTable(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub tags: Vec<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone)]
pub struct Feature {
    pub tags: Vec<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub background: Vec<Step>,
    pub scenarios: Vec<Scenario>,
    pub scenario_outlines: Vec<ScenarioOutline>,
}

impl Feature {
    pub fn scenarios(&self) -> impl Iterator<Item = Scenario> + '_ {
        let scenarios = self.scenarios.clone();

        let outline_scenarios = self.scenario_outlines.iter().flat_map(|e| e.scenarios());

        scenarios.into_iter().chain(outline_scenarios)
    }

    pub fn total_scenario_count(&self) -> usize {
        let in_outlines: usize = self
            .scenario_outlines
            .iter()
            .map(|s| {
                let v: usize = s.scenarios.iter().map(|s| s.len()).sum();
                v
            })
            .sum();

        self.scenarios.len() + in_outlines
    }
}
