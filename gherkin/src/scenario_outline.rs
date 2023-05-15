use crate::{Scenario, Step};

#[derive(Debug, Clone, PartialEq)]
pub struct TaggedScenarios {
    tags: Vec<String>,
    placeholders: Vec<String>,
    values: Vec<Vec<String>>,
}

impl TaggedScenarios {
    pub fn new(
        tags: Vec<String>,
        placeholders: Vec<String>,
        values: Vec<Vec<String>>,
    ) -> Result<Self, String> {
        if values.iter().all(|v| v.len() == placeholders.len()) {
            Ok(Self {
                tags,
                placeholders,
                values,
            })
        } else {
            Err("".into())
        }
    }

    pub fn index_of(&self, placeholder: &str) -> Option<usize> {
        self.placeholders.iter().enumerate().find_map(|(idx, p)| {
            if p == placeholder {
                Some(idx)
            } else {
                None
            }
        })
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioOutline {
    pub tags: Vec<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Vec<Step>,
    pub scenarios: Vec<TaggedScenarios>,
}

impl ScenarioOutline {
    pub fn scenarios(&self) -> impl Iterator<Item = Scenario> + '_ {
        self.scenarios.iter().flat_map(|s| {
            s.values.iter().map(|row| {
                let steps = self.steps.clone().into_iter().map(|mut step| {
                    row.iter().enumerate().for_each(|(idx, cell)| {
                        let placeholder = &s.placeholders[idx];
                        let from = &format!("<{placeholder}>");
                        let to = cell;
                        step.description = step.description.replace(from, to);

                        if let Some(data) = &mut step.data {
                            data.replace(from, to);
                        }
                    });
                    step
                });

                Scenario {
                    tags: s.tags.clone(),
                    name: self.name.clone(),
                    description: self.description.clone(),
                    steps: steps.collect(),
                }
            })
        })
    }
}

#[test]
fn scenario_outline() {
    use crate::{StepData, StepType};

    let outline = ScenarioOutline {
        tags: Vec::new(),
        name: None,
        description: None,
        steps: vec![
            Step::new(StepType::Given, "some <text>".into(), None),
            Step::new(
                StepType::Then,
                "the following text".into(),
                Some(StepData::DocString(
                    "The text <extra_text>\nwith some extra bass".into(),
                )),
            ),
        ],
        scenarios: vec![
            TaggedScenarios::new(
                Vec::new(),
                vec!["extra_text".into(), "text".into()],
                vec![vec!["extra hihi".into(), "hihi".into()]],
            )
            .unwrap(),
            TaggedScenarios::new(
                Vec::new(),
                vec!["text".into(), "extra_text".into()],
                vec![vec!["hehe".into(), "extra hehe".into()]],
            )
            .unwrap(),
            TaggedScenarios::new(
                Vec::new(),
                vec!["text".into(), "extra_text".into()],
                vec![vec!["hoho".into(), "extra hoho".into()]],
            )
            .unwrap(),
        ],
    };

    let scenarios: Vec<_> = outline.scenarios().collect();

    fn make_scenario(name: &str) -> Scenario {
        Scenario {
            tags: Vec::new(),
            name: None,
            description: None,
            steps: vec![
                Step::new(StepType::Given, format!("some {name}"), None),
                Step::new(
                    StepType::Then,
                    "the following text".into(),
                    Some(StepData::DocString(format!(
                        "The text extra {name}\nwith some extra bass"
                    ))),
                ),
            ],
        }
    }

    let expected = vec![
        make_scenario("hihi"),
        make_scenario("hehe"),
        make_scenario("hoho"),
    ];

    assert_eq!(expected, scenarios);
}
